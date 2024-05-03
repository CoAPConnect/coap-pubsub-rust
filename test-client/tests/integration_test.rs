use std::process::{Command, Child, Stdio};
use std::io::{BufRead, BufReader, Write};
use std::error::Error;
use tokio::time::{sleep, Duration};

pub struct CoapTestClient {
    process: Child,
    stdin: std::process::ChildStdin,
    reader: BufReader<std::process::ChildStdout>,
}

impl CoapTestClient {
    /// Starts the CoAP client process and prepares it for interaction.
    pub async fn start() -> Result<Self, Box<dyn Error>> {
        let mut process = Command::new("target/debug/client")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let stdin = process.stdin.take().ok_or("Failed to open stdin")?;
        let stdout = process.stdout.take().ok_or("Failed to open stdout")?;

        Ok(Self {
            process,
            stdin,
            reader: BufReader::new(stdout),
        })
    }

    pub async fn send_command(&mut self, command: &str) -> Result<(), Box<dyn Error>> {
        writeln!(self.stdin, "{}", command)?;
        self.stdin.flush()?;
		sleep(Duration::from_millis(100)).await;
        Ok(())
    }
 
	pub async fn read_all_output(&mut self) -> Result<String, Box<dyn Error>> {
		let mut output = String::new();
		let mut prompt_count = 0;
		loop {
			let mut line = String::new();
			let bytes_read = self.reader.read_line(&mut line)?;
			output.push_str(&line);
			if line.contains('\n'){
			break;
			}
		}
		Ok(output)
	}


}
#[tokio::test]
async fn test_full_coap_pubsub_workflow() -> Result<(), Box<dyn Error>> {
    let mut client = CoapTestClient::start().await?;

    // 1. Verify no topics exist initially
    client.send_command("1").await?;
	let output = client.read_all_output().await?;
	println!("Debug output: {:?}", output);
    assert!(response_contains(&output, "[]"));

    // 10. Perform initial topic-configuration discovery
    client.send_command("10").await?;
    let output = client.read_all_output().await?;
	println!("Debug output: {:?}", output);
    assert!(response_contains(&output, "Response:"));

    // 11. Perform initial topic-data discovery
    client.send_command("11").await?;
    let output = client.read_all_output().await?;
	println!("Debug output: {:?}", output);
    assert!(response_contains(&output, "Response:"));

    // 12. Perform initial topic collection discovery
    client.send_command("12").await?;
    let output = client.read_all_output().await?;
	println!("Debug output: {:?}", output);
    assert!(response_contains(&output, "core.ps.coll"));

    // 4. Create a new topic named "weather"
    client.send_command("4 weather").await?;
    let creation_output = client.read_all_output().await?;
	println!("Debug output: {:?}", creation_output);
    assert!(response_contains(&creation_output, "2.01")); 
    let data_uri = extract_data_uri(&creation_output);
    let topic_uri = extract_topic_uri(&creation_output);

    // 1. Verify the topic "weather" exists
    client.send_command("1").await?;
    let output = client.read_all_output().await?;
	println!("Debug output: {:?}", output);
    assert!(response_contains(&output, "weather"));

    // 5. Create (update) the topic data with JSON payload
    let payload = format!("5 {} {{\"temperature\":24}}", data_uri);
    println!("Sending payload: {}", payload);   
    client.send_command(&payload).await?;
    let output = client.read_all_output().await?;
	println!("Debug output: {:?}", output);
    assert!(response_contains(&output, "Created"));

    // 2. Subscribe to the new topic
    client.send_command(&format!("2 {}", data_uri)).await?;
    let output = client.read_all_output().await?;
	println!("Debug output: {:?}", output);
    assert!(response_contains(&output, "\"temperature\":24"));

    // 3. Unsubscribe
    client.send_command(&format!("3 {}", data_uri)).await?;
    let output = client.read_all_output().await?;
	println!("Debug output: {:?}", output);
    assert!(response_contains(&output, "Unsubscribe"));

    // Update topic data with a new temperature value
    client.send_command(&format!("5 {} {{\"temperature\":25}}", data_uri)).await?;
    let output = client.read_all_output().await?;
	println!("Debug output: {:?}", output);
    assert!(response_contains(&output, "Updated"));

    // 9. Read the latest data directly
    client.send_command(&format!("9 {}", data_uri)).await?;
    let output = client.read_all_output().await?;
	println!("Debug output: {:?}", output);
    assert!(response_contains(&output, "25"));

    // 6. Delete the topic
    let topic_uri = extract_topic_uri(&creation_output);
    client.send_command(&format!("6 {}", topic_uri)).await?;
    let output = client.read_all_output().await?;
	println!("Debug output: {:?}", output);

    // 1. Verify the topic "weather" no longer exists
    client.send_command("1").await?;
    let output = client.read_all_output().await?;
    println!("Debug output: {:?}", output);
    assert!(response_contains(&output, "[]"));

    Ok(())
}

fn response_contains(output: &str, expected: &str) -> bool {
    output.contains(expected)
}

fn validate_temperature_update(output: &str, expected_temp: i32) -> bool {
    let v: serde_json::Value = serde_json::from_str(output).unwrap_or_default();
    if let Some(num) = v["temperature"].as_i64() {
        num == expected_temp as i64
    } else {
        false
    }
}

fn extract_data_uri(output: &str) -> String {
    output.lines()
        .find_map(|line| {
            if let Some(start) = line.find('{') {
                let end = line.rfind('}').map_or(line.len(), |i| i + 1);
                let json_str = &line[start..end];
                match serde_json::from_str::<serde_json::Value>(json_str) {
                    Ok(json) => json["topic-data"].as_str().and_then(|s| s.split('/').last()).map(String::from),
                    Err(e) => {
                        eprintln!("Failed to parse JSON: {:?}", e);
                        None
                    }
                }
            } else {
                None
            }
        })
        .unwrap_or_default()
}

fn extract_topic_uri(output: &str) -> String {
    output.lines()
        .find_map(|line| {
            if let Some(start) = line.find('{') {
                let end = line.rfind('}').map_or(line.len(), |i| i + 1);
                let json_str = &line[start..end];
                match serde_json::from_str::<serde_json::Value>(json_str) {
                    Ok(json) => json["Location-Path"].as_str().and_then(|s| s.split('/').last()).map(String::from),
                    Err(e) => {
                        eprintln!("Failed to parse JSON: {:?}", e);
                        None
                    }
                }
            } else {
                None
            }
        })
        .unwrap_or_default()
}