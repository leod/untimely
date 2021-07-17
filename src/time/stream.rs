use super::{Time, TimeTag, Samples};

pub fn predict_stream_time<InputTag, StreamTag>(
    samples: &Samples<InputTag, Time<StreamTag>>,
    current_source_time: Time<InputTag>,
) -> Time<StreamTag>
where
    InputTag: TimeTag,
    StreamTag: TimeTag,
{
    let time_pairs: Vec<(f64, f64)> = samples
        .iter()
        .map(|(source_time, stream_time)| (source_time.to_secs(), stream_time.to_secs()))
        .collect();
    let time_pairs = pareen::slice(&time_pairs);

    let slope = 1.0;
    let regression_line = pareen::simple_linear_regression_with_slope(slope, time_pairs);

    Time::from_secs(regression_line.eval(current_source_time.to_secs()))
}