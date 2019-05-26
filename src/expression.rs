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
    pub day: &'a str,
    pub month: &'a str,
    pub command: &'a str,
}

impl<'a> fmt::Display for Expression<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {}",
            self.minute, self.hour, self.date, self.day, self.month, self.command
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
    /// let e = Expression::new("* * * ? * command");
    /// assert_eq!(e, Expression {
    ///   minute: "*",
    ///   hour: "*",
    ///   date: "*",
    ///   month: "?",
    ///   day: "*",
    ///   command: "command",
    /// });
    /// ```
    pub fn new(expression_str: &str) -> Expression {
        let spw: Vec<&str> = expression_str.split_whitespace().collect();
        if spw.len() != 6 {
            panic!("Invalid expression.")
        }
        Expression {
            minute: spw[0],
            hour: spw[1],
            date: spw[2],
            month: spw[3],
            day: spw[4],
            command: spw[5],
        }
    }
}

/// # Examples
/// ```
/// use cron_gate::expression;
///
/// let v = expression::parse_minute("1,2,30").unwrap();
/// assert_eq!(v, vec![1, 2, 30]);
/// ```
pub fn parse_minute(minute: &str) -> Result<Vec<i8>, String> {
    let mut minutes: Vec<i8> = Vec::new();
    let units = minute.split(',');
    for u in units {
        match parse_unit(u, 0, 59) {
            Ok(mut v) => minutes.append(&mut v),
            Err(e) => return Err(format!("Invalid expression on '{}': {}", u, e)),
        };
    }
    Ok(minutes)
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
/// let v = expression::parse_unit("2-4", 0, 5).unwrap();
/// assert_eq!(v, vec![2, 3, 4]);
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

    if unit == "*" {
        for i in min..(max + 1) {
            ret.push(i);
        }
    } else {
        let re = Regex::new(r"^(\d)-(\d)$").unwrap();
        match re.captures(unit) {
            Some(caps) => match parse_range(caps, min, max) {
                Ok(mut v) => ret.append(&mut v),
                Err(e) => return Err(e),
            },
            None => {
                match unit.parse::<i8>() {
                    Ok(n) => {
                        if n < min || max < n {
                            return Err(format!("invalid range '{}'", unit));
                        }
                        ret.push(n);
                    }
                    Err(_) => return Err(format!("cannot parse '{}'", unit)),
                };
            }
        }
    }
    Ok(ret)
}

fn parse_range(caps: Captures, min: i8, max: i8) -> Result<Vec<i8>, String> {
    let mut ret: Vec<i8> = Vec::new();
    let ranmge_min: i8;
    let ranmge_max: i8;
    match &caps[1].parse::<i8>() {
        Ok(n) => {
            ranmge_min = *n;
        }
        Err(_) => return Err(format!("cannot parse '{}'", &caps[1])),
    };
    match &caps[2].parse::<i8>() {
        Ok(n) => {
            ranmge_max = *n;
        }
        Err(_) => return Err(format!("cannot parse '{}'", &caps[2])),
    };

    if ranmge_min > ranmge_max {
        return Err(format!(
            "left side cannot be greater than right one: {}",
            &caps[0]
        ));
    }

    if ranmge_min < min {
        return Err(format!("invalid range: {}", ranmge_min));
    }

    if max < ranmge_max {
        return Err(format!("invalid range: {}", ranmge_max));
    }

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
    fn test_parse_unit() {
        assert_eq!(uniq_and_sort(&vec![1, 1, 2, 2, 3]), vec![1, 2, 3]);
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
}
