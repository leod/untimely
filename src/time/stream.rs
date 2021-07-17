use super::{Samples, Time, TimeTag};

pub fn predict_stream_time<InputTag, StreamTag>(
    stream_samples: &Samples<InputTag, Time<StreamTag>>,
    current_input_time: Time<InputTag>,
) -> Option<Time<StreamTag>>
where
    InputTag: TimeTag,
    StreamTag: TimeTag,
{
    if stream_samples.len() < 2 {
        return None;
    }

    let time_pairs: Vec<(f64, f64)> = stream_samples
        .iter()
        .map(|(source_time, stream_time)| (source_time.to_secs(), stream_time.to_secs()))
        .collect();
    let time_pairs = pareen::slice(&time_pairs);

    let slope = 1.0;
    let regression_line = pareen::simple_linear_regression_with_slope(slope, time_pairs);

    Some(Time::from_secs(
        regression_line.eval(current_input_time.to_secs()),
    ))
}
