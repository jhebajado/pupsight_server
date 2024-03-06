use std::sync::Mutex;

use actix_multipart::{Field, Multipart};
use actix_web::{
    web::{self, Data},
    App, Error, HttpResponse, HttpServer,
};
use dotenvy::dotenv;
use futures::{StreamExt, TryStreamExt};
use image::{imageops::FilterType, GenericImageView};
use ndarray::{s, Array, Axis};
use ort::{tensor::InputTensor, ExecutionProvider, InMemorySession, Session};

fn main() -> std::io::Result<()> {
    dotenv().ok();

    let environment = ort::Environment::builder()
        .with_log_level(ort::LoggingLevel::Verbose)
        .with_execution_providers([ExecutionProvider::cuda()])
        .build()
        .unwrap()
        .into_arc();

    let session = ort::SessionBuilder::new(&environment)
        .unwrap()
        .with_optimization_level(ort::GraphOptimizationLevel::Level3)
        .unwrap()
        .with_intra_threads(1)
        .unwrap()
        .with_model_from_memory(include_bytes!("../model.onnx"))
        .unwrap();

    let session_data = Data::new(Mutex::new(session));

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(start(session_data))
}

async fn start(session: Data<Mutex<InMemorySession<'static>>>) -> std::io::Result<()> {
    let server_url = std::env::var("SERVER_URL").expect("SERVER_URL must be set");

    println!("SERVER_URL: {server_url}");

    HttpServer::new(move || App::new().app_data(session.clone()).service(process_image))
        .bind(server_url)?
        .run()
        .await
}

#[actix_web::post("/scan")]
async fn process_image(
    (session, mut payload): (web::Data<Mutex<InMemorySession<'static>>>, Multipart),
) -> Result<HttpResponse, Error> {
    println!("Proccessing image");
    if let Ok(Some(mut field)) = payload.try_next().await {
        let file_data = get_field_filedata(&mut field).await?;

        let image = {
            let raw = image::load_from_memory(&file_data).unwrap();
            let (width, height) = raw.dimensions();
            let size = width.min(height);
            let (center_x, center_y) = (width / 2, height / 2);
            let (x, y) = (center_x - size / 2, center_y - size / 2);

            raw.crop_imm(x, y, width, height)
                .resize_exact(640, 640, FilterType::CatmullRom)
        };

        let mut input = Array::zeros((1, 3, 640, 640)).into_dyn();
        for pixel in image.pixels() {
            let x = pixel.0 as _;
            let y = pixel.1 as _;
            let [r, g, b, _] = pixel.2 .0;
            input[[0, 0, y, x]] = (r as f32) / 255.0;
            input[[0, 1, y, x]] = (g as f32) / 255.0;
            input[[0, 2, y, x]] = (b as f32) / 255.0;
        }

        println!("Inference start");

        let output = {
            let model = session.lock().unwrap();

            model
                .run([InputTensor::FloatTensor(input)])
                .unwrap()
                .first()
                .unwrap()
                .try_extract::<f32>()
                .unwrap()
                .view()
                .t()
                .to_owned()
        };

        println!("Inference success");
        let mut boxes = Vec::new();
        let output = output.slice(s![.., .., 0]);
        for row in output.axis_iter(Axis(0)) {
            let row: Vec<_> = row.iter().copied().collect();
            let (class_id, probability) = row
                .iter()
                .skip(4)
                .enumerate()
                .map(|(index, value)| (index, *value))
                .reduce(|a, row| if row.1 > a.1 { row } else { a })
                .unwrap();
            if probability < 0.3 {
                continue;
            }
            let classification = Classification::from(class_id);
            let xc = row[0];
            let yc = row[1];
            let w = row[2];
            let h = row[3];
            let x1 = xc - w / 2.0;
            let x2 = xc + w / 2.0;
            let y1 = yc - h / 2.0;
            let y2 = yc + h / 2.0;
            boxes.push(OutputBox {
                x1,
                y1,
                x2,
                y2,
                classification,
                probability,
            });
        }

        boxes.sort_by(|box1, box2| box2.probability.total_cmp(&box1.probability));
        let mut result = Vec::new();
        while !boxes.is_empty() {
            let first = boxes[0];
            result.push(first);
            boxes.retain(|box1| iou(&first, box1) < 0.75)
        }

        return Ok(HttpResponse::Ok()
            .content_type("application/json")
            .json(result));
    }

    Ok(HttpResponse::NotAcceptable().finish())
}

pub async fn get_field_filedata(field: &mut Field) -> Result<Vec<u8>, Error> {
    let mut buffer = Vec::<u8>::new();

    while let Some(chunk) = field.next().await {
        let data = chunk.unwrap();
        buffer.extend_from_slice(&data);
    }

    Ok(buffer)
}

#[derive(Clone, Copy, Debug, serde::Serialize)]
pub enum Classification {
    Normal,
    Incipient,
    Mature,
    Hypermature,
}

#[derive(Clone, Copy, Debug, serde::Serialize)]
pub struct OutputBox {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    probability: f32,
    classification: Classification,
}

impl From<usize> for Classification {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::Hypermature,
            1 => Self::Incipient,
            2 => Self::Mature,
            3 => Self::Normal,
            _ => panic!("Invalid numerical value for classification"),
        }
    }
}

fn iou(box1: &OutputBox, box2: &OutputBox) -> f32 {
    intersection(box1, box2) / union(box1, box2)
}

fn union(box1: &OutputBox, box2: &OutputBox) -> f32 {
    let box1_area = (box1.x2 - box1.x1) * (box1.y2 - box1.y1);
    let box2_area = (box2.x2 - box2.x1) * (box2.y2 - box2.y1);
    box1_area + box2_area - intersection(box1, box2)
}

fn intersection(box1: &OutputBox, box2: &OutputBox) -> f32 {
    let x1 = box1.x1.max(box2.x1);
    let y1 = box1.y1.max(box2.y1);
    let x2 = box1.x2.min(box2.x2);
    let y2 = box1.y2.min(box2.y2);
    (x2 - x1) * (y2 - y1)
}
