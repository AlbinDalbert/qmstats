use std::{collections::HashMap, thread, time};
use wmi::{WMIConnection, COMLibrary,Variant};
use anyhow;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn establish_wmi_connection() {

        thread::spawn(|| {
            
            match init_wmi_connection() {
                Ok(_) => assert!(true),
                Err(_) => assert!(false),
            }

        });
    }

    #[test]
    fn check_temp_with_arg() {

        thread::spawn(|| {
            
            let wmi = match init_wmi_connection() {
                Ok(wmi) => wmi,
                Err(_) => panic!("panic"),
            };
            // for _ in 1..100 {
    
                let temp = get_temp(&wmi);
                println!("current temp: {}", temp);
                
            // }
            assert!(true);
            
        });
    }
}


pub fn init_wmi_connection() -> Result<WMIConnection, anyhow::Error>{
    
    let com_lib = COMLibrary::new()?;

    let wmi_con = WMIConnection::new(com_lib.into())?;

    Ok(wmi_con)
}

pub fn get_temp(wmi: &WMIConnection) -> f64 {

    let results: Vec<HashMap<String, Variant>> = wmi
    .raw_query(
        "SELECT * FROM Win32_PerfFormattedData_Counters_ThermalZoneInformation",
    )
    .unwrap();
    
    println!("{:?}", results);

    let data = results.get(0).unwrap();

    let kelvin: f64 = match data.get("Temperature").unwrap() {
        Variant::UI4(val) => *val as f64,
        _ => 0.0,
    };
    kelvin - 273.0
}