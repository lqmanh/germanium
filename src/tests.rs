use super::{is_valid_address, read_trap, TransportAddress, VarBind};
use std::io::Result as IOResult;

#[test]
fn test_is_valid_address_ok() {
    match is_valid_address("https://google.com".to_string()) {
        Err(err) => panic!(err),
        Ok(_) => (),
    };
}

#[test]
#[should_panic(expected = "Invalid URL scheme: ws")]
fn test_is_valid_address_invalid_url_scheme() {
    match is_valid_address("ws://google.com".to_string()) {
        Err(err) => panic!(err),
        Ok(_) => (),
    };
}

#[test]
#[should_panic(expected = "Invalid URL: Hello world")]
fn test_is_valid_address_invalid_url() {
    match is_valid_address("Hello world".to_string()) {
        Err(err) => panic!(err),
        Ok(_) => (),
    };
}

#[test]
fn test_read_trap_ok() -> IOResult<()> {
    let buffer = b"localhost
        UDP: [127.0.0.1]:42935->[127.0.0.1]:162
        DISMAN-EVENT-MIB::sysUpTimeInstance 1:5:47:16.76
        SNMPv2-MIB::snmpTrapOID.0 NET-SNMP-EXAMPLES-MIB::netSnmpExampleHeartbeatNotification
        NET-SNMP-EXAMPLES-MIB::netSnmpExampleHeartbeatRate 123456" as &[u8];

    let trap = read_trap(buffer)?;

    assert_eq!(trap.remote_hostname, "localhost");
    assert_eq!(
        trap.transport_address,
        TransportAddress {
            protocol: "UDP".to_string(),
            remote_address: "[127.0.0.1]:42935".to_string(),
            local_address: "[127.0.0.1]:162".to_string(),
        },
    );
    assert_eq!(
        trap.varbinds,
        [
            VarBind {
                oid: "DISMAN-EVENT-MIB::sysUpTimeInstance".to_string(),
                value: "1:5:47:16.76".to_string(),
            },
            VarBind {
                oid: "SNMPv2-MIB::snmpTrapOID.0".to_string(),
                value: "NET-SNMP-EXAMPLES-MIB::netSnmpExampleHeartbeatNotification".to_string(),
            },
            VarBind {
                oid: "NET-SNMP-EXAMPLES-MIB::netSnmpExampleHeartbeatRate".to_string(),
                value: "123456".to_string(),
            },
        ]
    );

    Ok(())
}

#[test]
#[should_panic]
fn test_read_trap_malformed_input() {
    let buffer = b"Hello world" as &[u8];
    match read_trap(buffer) {
        Err(err) => panic!(err),
        Ok(_) => (),
    };
}
