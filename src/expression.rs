extern crate chrono;
extern crate regex;

use chrono::offset::TimeZone;
use chrono::{DateTime, Datelike, Duration, Local, ParseError, Timelike, Weekday};
use regex::Captures;
use regex::Regex;
use std::collections::HashSet;
use std::fmt;

pub const DATE_FORMAT: &str = "%Y/%m/%d %H:%M";

#[derive(Debug, PartialEq, Clone)]
pub struct CronLine {
    pub datetime: DateTime<Local>,
    pub command: String,
}

impl fmt::Display for CronLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {}",
            self.datetime.format("%Y/%m/%d %H:%M"),
            self.command
        )
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Expression {
    pub minute: String,
    pub hour: String,
    pub date: String,
    pub month: String,
    pub day: String,
    pub command: String,
    pub minute_vec: Vec<u32>,
    pub hour_vec: Vec<u32>,
    pub date_vec: Vec<u32>,
    pub month_vec: Vec<u32>,
    pub day_vec: Vec<u32>,
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {}",
            self.minute, self.hour, self.date, self.month, self.day, self.command
        )
    }
}

impl Expression {
    /// Returns a Expression
    ///
    /// # Examples
    ///
    /// ```
    /// use cron_gate::expression::Expression;
    ///
    /// let e = Expression::new("1 2 3 4 5 command").unwrap();
    /// assert_eq!(e, Expression {
    ///   minute: "1".to_string(),
    ///   hour: "2".to_string(),
    ///   date: "3".to_string(),
    ///   month: "4".to_string(),
    ///   day: "5".to_string(),
    ///   command: "command".to_string(),
    ///   minute_vec: vec![1],
    ///   hour_vec: vec![2],
    ///   date_vec: vec![3],
    ///   month_vec: vec![4],
    ///   day_vec: vec![5],
    /// });
    /// ```
    pub fn new(expression_str: &str) -> Result<Expression, String> {
        let spw: Vec<&str> = expression_str.split_whitespace().collect();

        if spw.len() < 5 {
            return Err(format!("Invalid expression: {}", expression_str));
        }

        let mut command = "[command]".to_string();
        for i in 5..spw.len() {
            if i == 5 {
                command = spw[i].to_string();
            } else {
                command = format!("{} {}", command, spw[i]);
            }
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
            minute: spw[0].to_string(),
            hour: spw[1].to_string(),
            date: spw[2].to_string(),
            month: spw[3].to_string(),
            day: spw[4].to_string(),
            command: command,
            minute_vec,
            hour_vec,
            date_vec,
            month_vec,
            day_vec,
        })
    }

    /// Returns a vec of indexes of the datetime earliest from
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::Local;
    /// use chrono::offset::TimeZone;
    /// use cron_gate::expression::Expression;
    ///
    /// let e = Expression::new("1-7 3-6 2-5 3-4 3 command").unwrap();
    /// let from = Local.datetime_from_str("2019/5/4 3:2", "%Y/%m/%d %H:%M").unwrap();
    /// assert_eq!(e.earliest_date_time_index(from), [1, 0, 2, 2]);
    /// ```
    pub fn earliest_date_time_index(&self, from: DateTime<Local>) -> [usize; 4] {
        let mut ret = [0; 4];
        ret[0] = get_smalest_index_from(&self.minute_vec, from.minute());
        ret[1] = get_smalest_index_from(&self.hour_vec, from.hour());
        ret[2] = get_smalest_index_from(&self.date_vec, from.day());
        ret[3] = get_smalest_index_from(&self.month_vec, from.month());
        ret
    }

    /// Returns earler datetimes from
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::Local;
    /// use chrono::offset::TimeZone;
    /// use cron_gate::expression::Expression;
    ///
    /// let e = Expression::new("0 9 27-29 5 * command").unwrap();
    /// let from = Local.datetime_from_str("2019/5/28 0:0", "%Y/%m/%d %H:%M").unwrap();
    /// assert_eq!(e.earler_excuting_datetimes(from, 2), [
    ///   Local.datetime_from_str("2019/5/28 9:0", "%Y/%m/%d %H:%M").unwrap(),
    ///   Local.datetime_from_str("2019/5/29 9:0", "%Y/%m/%d %H:%M").unwrap(),
    /// ]);
    /// ```
    pub fn earler_excuting_datetimes(
        &self,
        from: DateTime<Local>,
        count: usize,
    ) -> Vec<DateTime<Local>> {
        let mut ret: Vec<DateTime<Local>> = vec![];
        let mut indexes = self.earliest_date_time_index(from);

        for year in (from.year() as i64)..((from.year() as i64) + 4 * (count as i64)) {
            if indexes[3] < self.month_vec.len() {
                for month_i in indexes[3]..self.month_vec.len() {
                    if indexes[2] < self.date_vec.len() {
                        let month = self.month_vec[month_i];
                        for date_i in indexes[2]..self.date_vec.len() {
                            if indexes[1] < self.hour_vec.len() {
                                let date = self.date_vec[date_i];
                                for hour_i in indexes[1]..self.hour_vec.len() {
                                    if indexes[0] < self.minute_vec.len() {
                                        let hour = self.hour_vec[hour_i];
                                        for minute_i in indexes[0]..self.minute_vec.len() {
                                            let minute = self.minute_vec[minute_i];
                                            match parse_datetime(year, month, date, hour, minute) {
                                                Ok(datetime) => {
                                                    if is_on_weekday(
                                                        &datetime.weekday(),
                                                        &self.day_vec,
                                                    ) {
                                                        ret.push(datetime);
                                                        if ret.len() >= count {
                                                            return ret;
                                                        }
                                                    }
                                                }
                                                Err(_) => { /* invalid date (e.g. 3/31) */ }
                                            }
                                        }
                                    }
                                    indexes[0] = 0;
                                }
                            }
                            indexes[0] = 0;
                            indexes[1] = 0;
                        }
                    }
                    indexes[0] = 0;
                    indexes[1] = 0;
                    indexes[2] = 0;
                }
            }
            indexes[0] = 0;
            indexes[1] = 0;
            indexes[2] = 0;
            indexes[3] = 0;
        }

        ret
    }

    /// Returns earler CronLines from
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::Local;
    /// use chrono::offset::TimeZone;
    /// use cron_gate::expression::{Expression, CronLine};
    ///
    /// let e = Expression::new("0 9 27-29 5 * command").unwrap();
    /// let from = Local.datetime_from_str("2019/5/28 0:0", "%Y/%m/%d %H:%M").unwrap();
    ///
    /// let result = e.executing_dates(from, 2);
    /// let expect = [
    ///     CronLine {
    ///         datetime: Local.datetime_from_str("2019/5/28 9:0", "%Y/%m/%d %H:%M").unwrap(),
    ///         command: "command".to_string(),
    ///     },
    ///     CronLine {
    ///         datetime: Local.datetime_from_str("2019/5/29 9:0", "%Y/%m/%d %H:%M").unwrap(),
    ///         command: "command".to_string(),
    ///     },
    /// ];
    /// assert_eq!(result, expect);
    /// ```
    pub fn executing_dates(&self, after: DateTime<Local>, number: usize) -> Vec<CronLine> {
        let mut vec: Vec<CronLine> = vec![];
        let datetimes = self.earler_excuting_datetimes(after, number);
        for datetime in datetimes {
            vec.push(CronLine {
                datetime,
                command: self.command.clone(),
            });
        }
        vec
    }
}

fn parse_datetime(
    year: i64,
    month: u32,
    date: u32,
    hour: u32,
    minute: u32,
) -> Result<DateTime<Local>, ParseError> {
    Local.datetime_from_str(
        &format!("{}/{}/{} {}:{}", year, month, date, hour, minute),
        DATE_FORMAT,
    )
}

fn is_on_weekday(weekday: &Weekday, v: &Vec<u32>) -> bool {
    match weekday {
        Weekday::Sun => v.into_iter().find(|&i| *i == 0 || *i == 7).is_some(),
        Weekday::Mon => v.into_iter().find(|&i| *i == 1).is_some(),
        Weekday::Tue => v.into_iter().find(|&i| *i == 2).is_some(),
        Weekday::Wed => v.into_iter().find(|&i| *i == 3).is_some(),
        Weekday::Thu => v.into_iter().find(|&i| *i == 4).is_some(),
        Weekday::Fri => v.into_iter().find(|&i| *i == 5).is_some(),
        Weekday::Sat => v.into_iter().find(|&i| *i == 6).is_some(),
    }
}

fn get_smalest_index_from(v: &Vec<u32>, from: u32) -> usize {
    for (index, i) in v.iter().enumerate() {
        if from <= *i {
            return index;
        }
    }
    v.len()
}

/// Returns the date range: from < x < to
///
/// # Examples
/// ```
/// extern crate chrono;
/// use chrono::{DateTime, Duration, Local};
/// use cron_gate::expression;
///
/// let from = Local::now();
/// let to = from + Duration::days(3);
/// let v = expression::get_date_range_between(from, to);
/// assert_eq!(v, vec![from + Duration::days(1), from + Duration::days(2)]);
/// ```
pub fn get_date_range_between(from: DateTime<Local>, to: DateTime<Local>) -> Vec<DateTime<Local>> {
    let mut ret = vec![];
    let mut current = from + Duration::days(1);
    while current < to {
        ret.push(current);
        current = current + Duration::days(1);
    }
    ret
}

/// # Examples
/// ```
/// use cron_gate::expression;
///
/// let v = expression::parse_block("1,4-6,2-12/3", 0, 59).unwrap();
/// assert_eq!(v, vec![1, 2, 4, 5, 6, 8, 11]);
/// ```
pub fn parse_block(minute: &str, min: u32, max: u32) -> Result<Vec<u32>, String> {
    let mut minutes: Vec<u32> = Vec::new();
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
pub fn parse_unit(unit: &str, min: u32, max: u32) -> Result<Vec<u32>, String> {
    let mut ret: Vec<u32> = Vec::new();

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
                    .parse::<u32>()
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

fn filter_interval(vec: &Vec<u32>, interval: u32) -> Vec<u32> {
    let mut ret = Vec::new();
    let from: u32;
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

fn parse_interval(unit: &str) -> u32 {
    let re = Regex::new(r".*/(\d*)$").unwrap();
    re.captures(unit)
        .map_or(1, |caps| caps[1].parse::<u32>().unwrap())
}

fn parse_range(caps: Captures, min: u32, max: u32) -> Result<Vec<u32>, String> {
    let ranmge_min = caps[1]
        .parse::<u32>()
        .map_err(|e| format!("Cannot parse '{}': {}", &caps[1], e))?;
    let ranmge_max = caps[2]
        .parse::<u32>()
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

    let mut ret: Vec<u32> = Vec::new();
    for i in ranmge_min..(ranmge_max + 1) {
        ret.push(i);
    }
    Ok(ret)
}

/// Returns Vec<u32> having unique and sorted values
fn uniq_and_sort(v: &Vec<u32>) -> Vec<u32> {
    let set: HashSet<_> = v.clone().drain(..).collect();
    let mut vec = vec![];
    vec.extend(set.into_iter());
    vec.sort_unstable();
    vec
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::offset::TimeZone;

    #[test]
    fn test_earler_excuting_datetimes() {
        let e = Expression::new("0 1-20/3 28 5 2 command").unwrap();
        let from = Local
            .datetime_from_str("2019/5/28 0:0", DATE_FORMAT)
            .unwrap();
        assert_eq!(
            e.earler_excuting_datetimes(from, 3),
            [
                Local
                    .datetime_from_str("2019/5/28 1:0", DATE_FORMAT)
                    .unwrap(),
                Local
                    .datetime_from_str("2019/5/28 4:0", DATE_FORMAT)
                    .unwrap(),
                Local
                    .datetime_from_str("2019/5/28 7:0", DATE_FORMAT)
                    .unwrap(),
            ]
        );
    }

    #[test]
    fn test_earler_excuting_datetimes_short_month() {
        let e = Expression::new("0 0 31 5-12 * command").unwrap();
        let from = Local
            .datetime_from_str("2019/5/1 0:0", DATE_FORMAT)
            .unwrap();
        assert_eq!(
            e.earler_excuting_datetimes(from, 3),
            [
                Local
                    .datetime_from_str("2019/5/31 0:0", DATE_FORMAT)
                    .unwrap(),
                Local
                    .datetime_from_str("2019/7/31 0:0", DATE_FORMAT)
                    .unwrap(),
                Local
                    .datetime_from_str("2019/8/31 0:0", DATE_FORMAT)
                    .unwrap(),
            ]
        );
    }

    #[test]
    fn test_earler_excuting_datetimes_new_year() {
        let e = Expression::new("0 0 * * * command").unwrap();
        let from = Local
            .datetime_from_str("2019/12/30 0:0", DATE_FORMAT)
            .unwrap();
        assert_eq!(
            e.earler_excuting_datetimes(from, 3),
            [
                Local
                    .datetime_from_str("2019/12/30 0:0", DATE_FORMAT)
                    .unwrap(),
                Local
                    .datetime_from_str("2019/12/31 0:0", DATE_FORMAT)
                    .unwrap(),
                Local
                    .datetime_from_str("2020/1/1 0:0", DATE_FORMAT)
                    .unwrap(),
            ]
        );
    }

    #[test]
    fn test_earler_excuting_datetimes_leap_year() {
        let e = Expression::new("0 0 29 2 * command").unwrap();
        let from = Local
            .datetime_from_str("2019/1/1 0:0", DATE_FORMAT)
            .unwrap();
        assert_eq!(
            e.earler_excuting_datetimes(from, 3),
            [
                Local
                    .datetime_from_str("2020/2/29 0:0", DATE_FORMAT)
                    .unwrap(),
                Local
                    .datetime_from_str("2024/2/29 0:0", DATE_FORMAT)
                    .unwrap(),
                Local
                    .datetime_from_str("2028/2/29 0:0", DATE_FORMAT)
                    .unwrap(),
            ]
        );
    }

    #[test]
    fn test_is_on_weekday() {
        let tue = Local
            .datetime_from_str("2019/5/28 0:0", DATE_FORMAT)
            .unwrap();
        assert!(is_on_weekday(&tue.weekday(), &vec![2]));
        assert!(!is_on_weekday(&tue.weekday(), &vec![0, 1, 3, 4, 5, 6, 7]));

        let sun = Local
            .datetime_from_str("2019/5/26 0:0", DATE_FORMAT)
            .unwrap();
        assert!(is_on_weekday(&sun.weekday(), &vec![0]));
        assert!(is_on_weekday(&sun.weekday(), &vec![7]));
        assert!(!is_on_weekday(&sun.weekday(), &vec![1, 2, 3, 4, 5, 6]));
    }

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
