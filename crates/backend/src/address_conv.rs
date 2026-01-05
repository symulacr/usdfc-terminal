#[cfg(feature = "ssr")]
use fvm_shared::address::{Address, Payload, Protocol};
#[cfg(feature = "ssr")]
use std::str::FromStr;

const EAM_NAMESPACE: u64 = 32;

#[cfg(feature = "ssr")]
pub fn evm_to_f4(evm: &str) -> Result<String, String> {
    let hex = evm.strip_prefix("0x").ok_or_else(|| "missing 0x prefix".to_string())?;
    let bytes = hex::decode(hex).map_err(|e| format!("invalid hex: {}", e))?;
    if bytes.len() != 20 {
        return Err("expected 20-byte EVM address".to_string());
    }
    let addr = Address::new_delegated(EAM_NAMESPACE, &bytes)
        .map_err(|e| format!("delegated address error: {}", e))?;
    Ok(addr.to_string())
}

#[cfg(feature = "ssr")]
pub fn f4_to_evm(f4: &str) -> Result<String, String> {
    let addr = Address::from_str(f4).map_err(|e| format!("invalid address: {}", e))?;
    if addr.protocol() != Protocol::Delegated {
        return Err("not a delegated (f4) address".to_string());
    }
    match addr.payload() {
        Payload::Delegated(d)
            if d.namespace() == EAM_NAMESPACE && d.subaddress().len() == 20 =>
        {
            Ok(format!("0x{}", hex::encode(d.subaddress())))
        }
        Payload::Delegated(_) => Err("unsupported delegated namespace or subaddress length".to_string()),
        _ => Err("unsupported address payload".to_string()),
    }
}

#[cfg(feature = "ssr")]
pub fn normalize_for_blockscout(input: &str) -> Result<String, String> {
    if input.starts_with("0x") {
        return Ok(input.to_string());
    }
    if input.starts_with("f4") {
        return f4_to_evm(input);
    }
    if input.starts_with("f1") || input.starts_with("f3") {
        return Err("f1/f3 addresses are not supported by Blockscout without conversion".to_string());
    }
    Err("unsupported address format".to_string())
}

 
