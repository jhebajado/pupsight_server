use image::{DynamicImage, GenericImageView};
use ndarray::{s, Array, Axis};
use ort::{tensor::InputTensor, Environment, ExecutionProvider, InMemorySession};
use tokio::sync::Mutex;

pub(crate) struct Detector {
    session: Mutex<InMemorySession<'static>>,
}

impl Detector {
    const MAX_BYTES_RECIP: f32 = 1.0 / 255.0;

    pub(crate) fn new() -> Self {
        let environment = Environment::builder()
            .with_log_level(ort::LoggingLevel::Verbose)
            .with_execution_providers([ExecutionProvider::cuda(), ExecutionProvider::onednn()])
            .build()
            .unwrap()
            .into_arc();

        let session = Mutex::new(
            ort::SessionBuilder::new(&environment)
                .unwrap()
                .with_optimization_level(ort::GraphOptimizationLevel::Level3)
                .unwrap()
                .with_intra_threads(1)
                .unwrap()
                .with_model_from_memory(include_bytes!("../model.onnx"))
                .unwrap(),
        );

        Self { session }
    }

    pub(crate) async fn infer(&self, image: &DynamicImage) -> Vec<ResultBox> {
        let mut input = Array::zeros((1, 3, 640, 640)).into_dyn();

        for pixel in image.pixels() {
            let x = pixel.0 as _;
            let y = pixel.1 as _;
            let [r, g, b, _] = pixel.2 .0;
            input[[0, 0, y, x]] = (r as f32) * Self::MAX_BYTES_RECIP;
            input[[0, 1, y, x]] = (g as f32) * Self::MAX_BYTES_RECIP;
            input[[0, 2, y, x]] = (b as f32) * Self::MAX_BYTES_RECIP;
        }

        let output = {
            let model = self.session.lock().await;

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

            boxes.push(OutputBox::new(
                row[0],
                row[1],
                row[2],
                row[3],
                probability,
                Classification::from(class_id),
            ));
        }

        boxes.sort_by(|box1, box2| box2.probability.total_cmp(&box1.probability));
        let mut result = Vec::<ResultBox>::new();
        while !boxes.is_empty() {
            let first = boxes[0];
            result.push(first.into_result_box());
            boxes.retain(|box1| Self::iou(&first, box1) < 0.75)
        }

        result
    }

    #[inline(always)]
    fn iou(a: &OutputBox, b: &OutputBox) -> f32 {
        let intersection = {
            let intersect_left = a.left.max(b.left);
            let intersect_top = a.top.max(b.top);
            let intersect_right = a.right.min(b.right);
            let intersect_bottom = a.bottom.min(b.bottom);

            let intersect_width = intersect_right - intersect_left;
            let intersect_height = intersect_bottom - intersect_top;

            intersect_width * intersect_height
        };

        let union = {
            let box1_area = a.width * a.height;
            let box2_area = b.width * b.height;

            box1_area + box2_area - intersection
        };

        intersection / union
    }
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub enum Classification {
    Normal,
    Incipient,
}

impl From<usize> for Classification {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::Normal,
            1 => Self::Incipient,
            _ => Self::Incipient,
        }
    }
}

#[derive(Clone, Copy, Debug, serde::Serialize)]
pub struct ResultBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub probability: f32,
    pub classification: Classification,
}

#[derive(Clone, Copy)]
pub struct OutputBox {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
    width: f32,
    height: f32,
    probability: f32,
    classification: Classification,
}

impl OutputBox {
    #[inline(always)]
    fn new(
        x_center: f32,
        y_center: f32,
        width: f32,
        height: f32,
        probability: f32,
        classification: Classification,
    ) -> Self {
        let w_half = width * 0.5;
        let h_half = height * 0.5;

        let left = x_center - w_half;
        let top = y_center - h_half;
        let right = x_center + w_half;
        let bottom = y_center + h_half;

        Self {
            left,
            top,
            right,
            bottom,
            width,
            height,
            probability,
            classification,
        }
    }

    #[inline(always)]
    fn into_result_box(self) -> ResultBox {
        ResultBox {
            x: self.left,
            y: self.top,
            width: self.width,
            height: self.height,
            probability: self.probability,
            classification: self.classification,
        }
    }
}
