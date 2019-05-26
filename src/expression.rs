extern crate regex;

use regex::Captures;
use regex::Regex;
use std::collections::HashSet;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub struct Expression<'a> {
    pub minute: &'a str,
    pub hour: &'a str,
    pub date: &'a str,
    pub month: &'a str,
    pub day: &'a str,
    pub command: &'a str,
    pub minute_vec: Vec<i8>,
    pub hour_vec: Vec<i8>,
    pub date_vec: Vec<i8>,
    pub month_vec: Vec<i8>,
    pub day_vec: Vec<i8>,
}

impl<'a> fmt::Display for Expression<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {}",
            self.minute, self.hour, self.date, self.month, self.day, self.command
        )
    }
}

impl<'a> Expression<'a> {
    /// Returns a Expression
    ///
    /// # Examples
    ///
    /// ```
    /// use cron_gate::expression::Expression;
    ///
    /// let e = Expression::new("1 2 3 4 5 command").unwrap();
    /// assert_eq!(e, Expression {
    ///   minute: "1",
    ///   hour: "2",
    ///   date: "3",
    ///   month: "4",
    ///   day: "5",
    ///   command: "command",
    ///   minute_vec: vec![1],
    ///   hour_vec: vec![2],
    ///   date_vec: vec![3],
    ///   month_vec: vec![4],
    ///   day_vec: vec![5],
    /// });
    /// ```
    pub fn new(expression_str: &str) -> Result<Expression, String> {
        let spw: Vec<&str> = expression_str.split_whitespace().collect();
        if spw.len() != 6 {
            panic!("Invalid expression.")
        }

        let minute_vec = parse_block(spw[0], 0, 59)
            .map_err(|e| format!("Error on minute: {}\n{}", spw[0], e))?;
        let hour_vec = parse_block(spw[1], 0, 23)
            .map_err(|e| format!("Error on hour: '{}'\n{}", spw[1], e))?;
        let date_vec = parse_block(spw[2], 1, 31)
            .map_err(|e| format!("Error on date: '{}'\n{}", spw[2], e))?;
        let month_vec = parse_block(spw[3], 1, 12)
            .map_err(|e| format!("Error on month: '{}'\n{}", spw[3], e))?;
        let day_vec =
            parse_block(spw[4], 0, 7).map_err(|e| format!("Error on day: '{}'\n{}", spw[4], e))?;

        Ok(Expression {
            minute: spw[0],
            hour: spw[1],
            date: spw[2],
            month: spw[3],
            day: spw[4],
            command: spw[5],
            minute_vec,
            hour_vec,
            date_vec,
            month_vec,
            day_vec,
        })
    }
}

/// # Examples
/// ```
/// use cron_gate::expression;
///
/// let v = expression::parse_block("1,4-6,2-12/3", 0, 59).unwrap();
/// assert_eq!(v, vec![1, 2, 4, 5, 6, 8, 11]);
/// ```
pub fn parse_block(minute: &str, min: i8, max: i8) -> Result<Vec<i8>, String> {
    let mut minutes: Vec<i8> = Vec::new();
    for u in minute.split(',') {
        minutes.append(
            &mut parse_unit(u, min, max)
                .map_err(|e| format!("Invalid expression on '{}'\n{}", u, e))?,
        );
    }
    Ok(uniq_and_sort(&minutes))
}

/// Returns numbers parsed from unit expression
///
/// # Examples
///
/// A number
/// ```
/// use cron_gate::expression;
///
/// let v = expression::parse_unit("1", 0, 3).unwrap();
/// assert_eq!(v, vec![1]);
/// ```
///
/// Wild card
/// ```
/// use cron_gate::expression;
///
/// let v = expression::parse_unit("*", 0, 3).unwrap();
/// assert_eq!(v, vec![0, 1, 2, 3]);
/// ```
///
/// Range
/// ```
/// use cron_gate::expression;
///
/// let v = expression::parse_unit("9-11", 0, 20).unwrap();
/// assert_eq!(v, vec![9, 10, 11]);
/// ```
///
/// Interval
/// ```
/// use cron_gate::expression;
///
/// let v1 = expression::parse_unit("5-11/3", 0, 20).unwrap();
/// assert_eq!(v1, vec![5, 8, 11]);
/// let v2 = expression::parse_unit("*/10", 0, 20).unwrap();
/// assert_eq!(v2, vec![0, 10, 20]);
/// ```
///
/// Error case
/// ```should_panic
/// use cron_gate::expression;
///
/// expression::parse_unit("a", 0, 3).unwrap();
/// ```
pub fn parse_unit(unit: &str, min: i8, max: i8) -> Result<Vec<i8>, String> {
    let mut ret: Vec<i8> = Vec::new();

    if unit.starts_with("*") {
        for i in min..(max + 1) {
            ret.push(i);
        }
    } else {
        let re = Regex::new(r"^(\d*)-(\d*)(/\d*)?$").unwrap();
        match re.captures(unit) {
            Some(caps) => {
                ret.append(&mut parse_range(caps, min, max)?);
            }
            None => {
                let n = unit
                    .parse::<i8>()
                    .map_err(|e| format!("Cannot parse '{}': {}", unit, e))?;
                if n < min || max < n {
                    return Err(format!(
                        "Invalid range '{}': should be in {} to {}",
                        unit, min, max
                    ));
                }
                ret.push(n);
            }
        }
    }

    Ok(filter_interval(&uniq_and_sort(&ret), parse_interval(unit)))
}

fn filter_interval(vec: &Vec<i8>, interval: i8) -> Vec<i8> {
    let mut ret = Vec::new();
    let from: i8;
    if vec.len() > 0 {
        from = vec[0];
    } else {
        from = 0;
    }
    for i in vec {
        if (i - from) % interval == 0 {
            ret.push(*i);
        }
    }
    ret
}

fn parse_interval(unit: &str) -> i8 {
    let re = Regex::new(r".*/(\d*)$").unwrap();
    re.captures(unit)
        .map_or(1, |caps| caps[1].parse::<i8>().unwrap())
}

fn parse_range(caps: Captures, min: i8, max: i8) -> Result<Vec<i8>, String> {
    let ranmge_min = caps[1]
        .parse::<i8>()
        .map_err(|e| format!("Cannot parse '{}': {}", &caps[1], e))?;
    let ranmge_max = caps[2]
        .parse::<i8>()
        .map_err(|e| format!("Cannot parse '{}': {}", &caps[2], e))?;

    if ranmge_min > ranmge_max {
        return Err(format!(
            "Left side cannot be greater than right one: {}",
            &caps[0]
        ));
    }

    if ranmge_min < min {
        return Err(format!("Invalid range: {}", ranmge_min));
    }

    if max < ranmge_max {
        return Err(format!("Invalid range: {}", ranmge_max));
    }

    let mut ret: Vec<i8> = Vec::new();
    for i in ranmge_min..(ranmge_max + 1) {
        ret.push(i);
    }
    Ok(ret)
}

/// Returns Vec<i8> having unique and sorted values
fn uniq_and_sort(v: &Vec<i8>) -> Vec<i8> {
    let set: HashSet<_> = v.clone().drain(..).collect();
    let mut vec = vec![];
    vec.extend(set.into_iter());
    vec.sort_unstable();
    vec
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_interval() {
        assert_eq!(filter_interval(&vec![0, 1, 2, 3, 4], 3), [0, 3]);
        assert_eq!(filter_interval(&vec![3, 4, 5, 6, 7], 2), [3, 5, 7]);
    }

    #[test]
    fn test_parse_interval() {
        assert_eq!(parse_interval("3"), 1);
        assert_eq!(parse_interval("1/2"), 2);
        assert_eq!(parse_interval("1/10"), 10);
    }

    #[test]
    fn test_parse_unit() {
        match parse_unit("0", 1, 4) {
            Ok(_) => assert!(false),
            Err(_) => assert!(true),
        }
        match parse_unit("1", 1, 4) {
            Ok(n) => assert_eq!(n, [1]),
            Err(_) => assert!(false),
        }
        match parse_unit("4", 1, 4) {
            Ok(n) => assert_eq!(n, [4]),
            Err(_) => assert!(false),
        }
        match parse_unit("5", 1, 4) {
            Ok(_) => assert!(false),
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn test_parse_range() {
        let re = Regex::new(r"^(\d)-(\d)$").unwrap();
        match parse_range(re.captures("1-3").unwrap(), 1, 3) {
            Ok(v) => assert_eq!(v, [1, 2, 3]),
            Err(_) => assert!(false),
        }
        match parse_range(re.captures("1-4").unwrap(), 1, 3) {
            Ok(_) => assert!(false),
            Err(_) => assert!(true),
        }
        match parse_range(re.captures("0-3").unwrap(), 1, 3) {
            Ok(_) => assert!(false),
            Err(_) => assert!(true),
        }
        match parse_range(re.captures("3-1").unwrap(), 1, 3) {
            Ok(_) => assert!(false),
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn test_uniq_and_sort() {
        assert_eq!(uniq_and_sort(&vec![1, 1, 2, 2, 3]), vec![1, 2, 3]);
    }
}
