use std::fs;

pub fn read_cpu_temp(sys_temp_path: &str) -> Result<String, String> {
    let response: String;
    let sys_cpu_temp: Result<String, _> = fs::read_to_string(sys_temp_path);
    match sys_cpu_temp {
        Ok(sys_cpu_temp) => {
            let cpu_temp: Result<i32, _> = sys_cpu_temp.trim().parse();
            match cpu_temp {
                Ok(cpu_temp) => {
                    let result = cpu_temp as f32 / 1000.0;
                    response = format!("{:.2}", result); // output is 4-sigfig Celcius. Example: "52.08"
                    return Ok(response);
                },
                Err(_) => {
                    response = format!("ERROR: failed to parse cpu temperature");
                    return Ok(response);
                }
            }
        },
        Err(_) => {
            response = format!("ERROR: failed to read cpu temperature from {}", sys_temp_path);
            return Err(response);
        }
    }
}