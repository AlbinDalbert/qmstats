use std::{collections::HashMap, thread, time};
use wmi::{WMIConnection, COMLibrary,Variant};
use anyhow;
mod test;


// initialize the WMI connection
pub fn init_wmi_connection() -> Result<WMIConnection, anyhow::Error>{
    
    let com_lib = COMLibrary::new()?;

    let wmi_con = WMIConnection::new(com_lib.into())?;

    Ok(wmi_con)
}

// get the temperature of the machine, returns in Celsius.
pub fn get_temp(wmi: &WMIConnection) -> f64 {

    let results: Vec<HashMap<String, Variant>> = wmi
    .raw_query(
        "SELECT Temperature FROM Win32_PerfFormattedData_Counters_ThermalZoneInformation",
    )
    .unwrap();

    let data = results.get(0).unwrap();

    let kelvin: f64 = match data.get("Temperature").unwrap() {
        Variant::UI4(val) => *val as f64,
        _ => -1.0,
    };
    kelvin - 273.0
}

// returns cpu utilization
// return -1 on error
pub fn get_cpu_util(wmi: &WMIConnection) -> f64 {
    let results: Vec<HashMap<String, Variant>> = wmi
    .raw_query(
        "SELECT PercentProcessorTime FROM Win32_PerfFormattedData_PerfOS_Processor",
    )
    .unwrap();

    let data = results.get(0).unwrap();

    let percent: f64 = match data.get("PercentProcessorTime").unwrap() {
        Variant::UI8(val) => *val as f64,
        _ => -1.0,
    };
    
    percent
}

// get available memory (ram) returns the volume in bytes

pub fn get_available_memory(wmi: &WMIConnection) -> f64 {

    let results: Vec<HashMap<String, Variant>> = wmi
    .raw_query(
        "SELECT AvailableBytes FROM Win32_PerfFormattedData_PerfOS_Memory",
    )
    .unwrap();

    let data = results.get(0).unwrap();

    let bytes: f64 = match data.get("AvailableBytes").unwrap() {
        Variant::UI8(val) => *val as f64,
        _ => 0.0,
    };

    bytes
}

// get the total amount of physical memory, returns in bytes

pub fn get_total_memory(wmi: &WMIConnection) -> f64 {

    let results: Vec<HashMap<String, Variant>> = wmi
    .raw_query(
        "SELECT TotalPhysicalMemory FROM Win32_ComputerSystem",
    )
    .unwrap();

    let data = results.get(0).unwrap();

    let bytes: f64 = match data.get("TotalPhysicalMemory").unwrap() {
        Variant::UI8(val) => *val as f64,
        _ => 0.0,
    };

    bytes
}