use std::env;
use std::sync::Once;

use crate::chart::Error;

static EPHE_INIT: Once = Once::new();

pub fn julian_day_from_datetime(year: i32, month: i32, day: i32, hour: f64) -> f64 {
    swiss_eph::safe::julday(year, month, day, hour)
}

pub fn datetime_from_julian_day(jd: f64) -> (i32, i32, i32, f64) {
    swiss_eph::safe::revjul(jd)
}

pub fn julian_day_from_ymdh(
    year: i32,
    month: i32,
    day: i32,
    hour: i32,
    minute: i32,
    second: i32,
) -> f64 {
    let hour_decimal = hour as f64 + minute as f64 / 60.0 + second as f64 / 3600.0;
    julian_day_from_datetime(year, month, day, hour_decimal)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DateTime {
    pub year: i32,
    pub month: i32,
    pub day: i32,
    pub hour: f64,
}

impl DateTime {
    pub fn new(year: i32, month: i32, day: i32, hour: f64) -> Self {
        Self {
            year,
            month,
            day,
            hour,
        }
    }

    pub fn from_ymdhms(
        year: i32,
        month: i32,
        day: i32,
        hour: i32,
        minute: i32,
        second: i32,
    ) -> Self {
        let hour_decimal = hour as f64 + minute as f64 / 60.0 + second as f64 / 3600.0;
        Self::new(year, month, day, hour_decimal)
    }

    pub fn to_julian_day(&self) -> f64 {
        julian_day_from_datetime(self.year, self.month, self.day, self.hour)
    }

    pub fn from_julian_day(jd: f64) -> Self {
        let (year, month, day, hour) = datetime_from_julian_day(jd);
        Self {
            year,
            month,
            day,
            hour,
        }
    }
}

pub struct Ephemeris {
    path: String,
    initialized: bool,
}

impl Ephemeris {
    pub fn new() -> Result<Self, Error> {
        Self::with_path(None)
    }

    pub fn with_path(path: Option<&str>) -> Result<Self, Error> {
        let path_str = path
            .map(|p| p.to_string())
            .or_else(|| env::var("SE_EPHE_PATH").ok())
            .unwrap_or_default();

        swiss_eph::safe::set_ephe_path(&path_str);

        Ok(Self {
            path: path_str,
            initialized: true,
        })
    }

    pub fn ensure_initialized() -> Result<(), Error> {
        EPHE_INIT.call_once(|| {
            let _ = Self::new();
        });
        Ok(())
    }

    pub fn close(&mut self) {
        if self.initialized {
            swiss_eph::safe::close();
            self.initialized = false;
        }
    }

    #[allow(dead_code)]
    pub fn path(&self) -> &str {
        &self.path
    }
}

impl Default for Ephemeris {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl Drop for Ephemeris {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ephemeris_init() {
        let eph = Ephemeris::new();
        assert!(eph.is_ok());
    }

    #[test]
    fn test_ephemeris_init_empty_path() {
        let eph = Ephemeris::with_path(None);
        assert!(eph.is_ok());
    }

    #[test]
    fn test_ephemeris_init_custom_path() {
        let eph = Ephemeris::with_path(Some("/custom/path"));
        assert!(eph.is_ok());
        assert_eq!(eph.unwrap().path(), "/custom/path");
    }

    #[test]
    fn test_ephemeris_env_override() {
        unsafe {
            env::set_var("SE_EPHE_PATH", "/env/path");
        }
        let eph = Ephemeris::with_path(None);
        assert!(eph.is_ok());
        assert_eq!(eph.unwrap().path(), "/env/path");
        unsafe {
            env::remove_var("SE_EPHE_PATH");
        }
    }

    #[test]
    fn test_ephemeris_close() {
        let mut eph = Ephemeris::new().unwrap();
        eph.close();
        assert!(!eph.initialized);
    }

    #[test]
    fn test_julian_day_from_datetime() {
        let jd = julian_day_from_datetime(2000, 1, 1, 12.0);
        assert!((jd - 2451545.0).abs() < 0.0001);
    }

    #[test]
    fn test_datetime_from_julian_day() {
        let (year, month, day, hour) = datetime_from_julian_day(2451545.0);
        assert_eq!(year, 2000);
        assert_eq!(month, 1);
        assert_eq!(day, 1);
        assert!((hour - 12.0).abs() < 0.0001);
    }

    #[test]
    fn test_julian_day_roundtrip() {
        let jd = julian_day_from_datetime(2024, 6, 15, 14.5);
        let (year, month, day, hour) = datetime_from_julian_day(jd);
        assert_eq!(year, 2024);
        assert_eq!(month, 6);
        assert_eq!(day, 15);
        assert!((hour - 14.5).abs() < 0.0001);
    }

    #[test]
    fn test_julian_day_from_ymdhms() {
        let jd = julian_day_from_ymdh(2000, 1, 1, 12, 0, 0);
        assert!((jd - 2451545.0).abs() < 0.0001);
    }

    #[test]
    fn test_datetime_struct() {
        let dt = DateTime::new(2000, 1, 1, 12.0);
        assert_eq!(dt.year, 2000);
        assert_eq!(dt.month, 1);
        assert_eq!(dt.day, 1);
        assert!((dt.hour - 12.0).abs() < f64::EPSILON);

        let jd = dt.to_julian_day();
        assert!((jd - 2451545.0).abs() < 0.0001);
    }

    #[test]
    fn test_datetime_from_ymdhms() {
        let dt = DateTime::from_ymdhms(2024, 6, 15, 14, 30, 45);
        let expected_hour = 14.0 + 30.0 / 60.0 + 45.0 / 3600.0;
        assert!((dt.hour - expected_hour).abs() < 0.0001);
    }

    #[test]
    fn test_datetime_struct_from_julian_day() {
        let dt = DateTime::from_julian_day(2451545.0);
        assert_eq!(dt.year, 2000);
        assert_eq!(dt.month, 1);
        assert_eq!(dt.day, 1);
        assert!((dt.hour - 12.0).abs() < 0.0001);
    }
}
