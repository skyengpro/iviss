use base64::Engine;

fn main() {
    // 1. What the backend gets from the environment (Base64 from deploy.sh)
    let env_key = "LS0tLS1CRUdJTiBQUklWQVRFIEtFWS0tLS0tCk1JSUV2UUlCQURBTkJna3Foa2lHOXcwQkFRRUZBQVNDQktjd2dnU2pBZ0VBQW9JQkFRQ2w5N1prSE5aalQ3UTQKa0VYT3k3TEtYQW1qanhRdEF0dnBxTEF3bFZTT2JVcGsxL3VsTEF2bUFRNHlJUkt1MzBCaHUrdHlxTEI5MlF3QgotLS0tLUVORCBQUklWQVRFIEtFWS0tLS0t";
    
    // 2. Our new resilience logic from JwtService::new
    let cleaned = env_key.trim_matches('"').trim();
    
    let raw_pem = if !cleaned.starts_with("-----") {
        match base64::engine::general_purpose::STANDARD.decode(cleaned.replace("\\n", "").replace("\n", "").trim()) {
            Ok(decoded) => String::from_utf8(decoded).unwrap_or_else(|_| cleaned.to_string()),
            Err(_) => cleaned.to_string(),
        }
    } else {
        cleaned.to_string()
    };

    let cleaned_pem = raw_pem.replace("\\n", "\n");
    
    println!("--- CLEANED PEM ---");
    println!("{}", cleaned_pem);
    
    if cleaned_pem.starts_with("-----BEGIN") && cleaned_pem.contains("MII") {
        println!("\n✅ RUST PARSING SUCCESS!");
    } else {
        println!("\n❌ RUST PARSING FAILURE!");
    }
}
