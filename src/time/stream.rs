use super::{LocalTime, Samples, Time, TimeTag};

pub fn predict_stream_time<StreamTag>(
    stream_samples: &Samples<Time<StreamTag>>,
    current_local_time: LocalTime,
) -> Option<Time<StreamTag>>
where
    StreamTag: TimeTag,
{
    if stream_samples.len() < 2 {
        return None;
    }

    let time_pairs: Vec<(f64, f64)> = stream_samples
        .iter()
        .map(|(local_time, stream_time)| (local_time.to_secs(), stream_time.to_secs()))
        .collect();
    let time_pairs = pareen::slice(&time_pairs);

    //let slope = 1.0;
    let regression_line = pareen::simple_linear_regression(time_pairs);

    Some(Time::from_secs(
        regression_line.eval(current_local_time.to_secs()),
    ))
}
