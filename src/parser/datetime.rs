use crate::error::{Error, Result};
use chrono::{DateTime, Utc, naive::Days, Months, NaiveDateTime, FixedOffset};

//From https://build.fhir.org/datatypes.html#dateTime
//YYYY                          2018          
//YYYY-MM                       1973-06
//YYYY-MM-DD                    1905-08-23
//YYYY-MM-DDThh:mm:ss+zz:zz     2015-02-07T13:28:17-05:00   2017-01-01T00:00:00.000Z

//TODO: TIMEZONE!!!!
//it is always saved in utc time
#[derive(Default)]
pub struct Fhir_DateTime {
    c: DateTime<Utc>,
}



impl Fhir_DateTime {

    pub fn from_string(s: &str) -> Result<Self> {
        if s.len() > 10 {
            if let Ok(c) = DateTime::parse_from_rfc3339(s) {
                Ok(Self {c: c.into()})
            } else {
                Err(Error::TimeStampParsingError)
            }
        } else {
            let mut parts = s.split("-").into_iter();
            if let Some(year) = parts.next() {
                    if let Some(dt) = chrono::naive::NaiveDate::from_ymd_opt(to_i32(year)?, 1, 1).unwrap().and_hms_opt(0, 0, 0) {
                        if let Some(month) = parts.next() {
                            if let Some(dt) = dt.checked_add_months(Months::new(to_u32(month)?-1)) {
                                if let Some(days) = parts.next() {
                                    if let Some(dt) = dt.checked_add_days(Days::new(to_u32(days)? as u64 -1)) {
                                        let dt = DateTime::<Utc>::from_utc(dt, Utc);
                                        Ok(Self {c:dt})
                                    } else {
                                        Err(Error::TimeStampParsingError)
                                    }
                                } else {
                                    let dt = DateTime::<Utc>::from_utc(dt, Utc);
                                    Ok(Self {c:dt})
                                }
                            } else {
                                Err(Error::TimeStampParsingError)
                            }
                        } else {
                            let dt = DateTime::<Utc>::from_utc(dt, Utc);
                            Ok(Self {c:dt})
                        }
                    } else {
                        Err(Error::TimeStampParsingError)
                    }
            } else { 
                Err(Error::TimeStampParsingError)
            }
        }
    }
    pub fn from_timestamp_millis(ts: i64) -> Result<Self> {
        if let Some(ndt) = NaiveDateTime::from_timestamp_millis(ts) {
            let dt = DateTime::<Utc>::from_utc(ndt, Utc);
            Ok(Self { c: dt })
        } else {
            Err(Error::TimeStampParsingError)
        }  
    }

    pub fn from_timestamp_bytes(ts: &[u8]) -> Result<Self> {
        let bytes: [u8; 8] = match ts.try_into() {
            Ok(v) => v,
            Err(_) => {return Err(Error::TimeStampParsingError)}
        };
        Fhir_DateTime::from_timestamp_millis(i64::from_be_bytes(bytes))
    }

    pub fn timestamp(&self) -> i64 {
        self.c.timestamp()
    }
    pub fn timestamp_millis(&self) -> i64 {
        self.c.timestamp_millis()
    }
    pub fn timestamp_bytes(&self) -> [u8; 8] {
        self.timestamp().to_be_bytes()
    }
    pub fn timestamp_millis_bytes(&self) -> [u8; 8] {
        self.timestamp_millis().to_be_bytes()
    }
} 


fn to_i32(s: &str) -> Result<i32> {
    match s.parse::<i32>() {
        Ok(parsed) => Ok(parsed),
        Err(_) => Err(Error::TimeStampParsingError)
    }
}

fn to_u32(s: &str) -> Result<u32> {
    match s.parse::<u32>() {
        Ok(parsed) => Ok(parsed),
        Err(_) => Err(Error::TimeStampParsingError)
    }
}




#[cfg(test)]
mod test {
    
    use super::*;

    #[test]
    fn fhir_dt_from_string_yyyy() {
        let conv = Fhir_DateTime::from_string("2018").unwrap();
        assert_eq!(conv.timestamp(), 1514764800);
        assert_eq!(conv.timestamp_millis(), 1514764800000);
        assert_eq!(conv.timestamp_bytes(),1514764800i64.to_be_bytes());
        assert_eq!(conv.timestamp_millis_bytes(),1514764800000i64.to_be_bytes());

        //let with_tz ="2015-02-07T13:28:17-05:00";
        //let with_ms = "2017-01-01T00:00:00.000Z";
    }

    #[test]
    fn fhir_dt_from_string_yyyy_mm() {
        let conv = Fhir_DateTime::from_string("1973-06").unwrap();
        assert_eq!(conv.timestamp(), 107740800);
        assert_eq!(conv.timestamp_millis(), 107740800000);
        assert_eq!(conv.timestamp_bytes(),107740800i64.to_be_bytes());
        assert_eq!(conv.timestamp_millis_bytes(),107740800000i64.to_be_bytes());
    }

    #[test]
    fn fhir_dt_from_string_yyyy_mm_dd() {
        let conv = Fhir_DateTime::from_string("1905-08-23").unwrap();
        assert_eq!(conv.timestamp(), -2031004800);
        assert_eq!(conv.timestamp_millis(),-2031004800000);
        assert_eq!(conv.timestamp_bytes(), (-2031004800i64).to_be_bytes());
        assert_eq!(conv.timestamp_millis_bytes(), (-2031004800000i64).to_be_bytes());
    }
    #[test]
    fn fhir_dt_from_string_rfc3339() {
        let conv_1 = Fhir_DateTime::from_string("2015-02-07T13:28:17-05:00").unwrap();
        let conv_2 = Fhir_DateTime::from_string("2017-01-01T00:00:00.000Z").unwrap();
        assert_eq!(conv_1.timestamp(),1423333697);
        assert_eq!(conv_2.timestamp_millis(),1483228800000);
    }

    #[test]
    fn fhir_dt_from_timestamp() {
        let ts = 1483228800000i64;
        let conv = Fhir_DateTime::from_timestamp_millis(ts).unwrap();
        assert_eq!(conv.c.to_rfc3339_opts(chrono::SecondsFormat::Millis, true), "2017-01-01T00:00:00.000Z".to_string());
        let conv_2 = Fhir_DateTime::from_timestamp_bytes(&ts.to_be_bytes()).unwrap();
        assert_eq!(conv_2.c.to_rfc3339_opts(chrono::SecondsFormat::Millis, true), "2017-01-01T00:00:00.000Z".to_string());
    }
}


   
