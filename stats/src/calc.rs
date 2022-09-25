use num::ToPrimitive;

pub fn average<T, C>(total: T, count: C) -> f64
where
    T: ToPrimitive,
    C: ToPrimitive,
{
    total.to_f64().unwrap() / count.to_f64().unwrap()
}

pub fn percent<N, T>(num: N, total: T) -> String
where
    N: ToPrimitive,
    T: ToPrimitive,
{
    format!("{:.2}%", (num.to_f64().unwrap() / total.to_f64().unwrap()) * 100_f64)
}
