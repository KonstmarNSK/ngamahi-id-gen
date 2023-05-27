use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::{self, general_purpose}, alphabet};
// use std::cell::RefCell;
// use std::thread;
//
// thread_local! {
//     static CLIENT: RefCell<f32> = RefCell::new(awc::Client);
// }


/*
    put value:
        http://localhost:2379/v3/kv/put POST
        RequestPut

    get value:
        http://localhost:2379/v3/kv/range POST
        RequestRange

    enlarge:
        http://localhost:2379/v3/kv/txn
        Transaction

            {
                "compare": [
                    {
                        "target": "VALUE",
                        "key": "dHgta2V5",
                        "value": "MA=="
                    }
                ],
                "success": [
                    {
                        "requestPut": {
                            "key": "dHgta2V5",
                            "value": "MQ=="
                        }
                    }
                ],
                "failure": [
                    {
                        "requestRange": {
                            "key": "dHgta2V5"
                        }
                    }
                ]
            }


    create seq:
        http://localhost:2379/v3/kv/txn
        Transaction

            {
                "compare": [
                    {
                        "version": "0",
                        "result": "EQUAL",
                        "target": "VERSION",
                        "key": "bm90LWV4aXN0ZW50"
                    }
                ],
                "success": [
                    {
                        "requestPut": {
                            "key": "bm90LWV4aXN0ZW50",
                            "value": "MA=="
                        }
                    }
                ],
                "failure": [
                    {
                        "requestRange": {
                            "key": "bm90LWV4aXN0ZW50"
                        }
                    }
                ]
            }




    comparison:

    message Compare {
      enum CompareResult {
        EQUAL = 0;
        GREATER = 1;
        LESS = 2;
        NOT_EQUAL = 3;
      }
      enum CompareTarget {
        VERSION = 0;
        CREATE = 1;
        MOD = 2;
        VALUE= 3;
      }
      CompareResult result = 1;
      // target is the key-value field to inspect for the comparison.
      CompareTarget target = 2;
      // key is the subject key for the comparison operation.
      bytes key = 3;
      oneof target_union {
        int64 version = 4;
        int64 create_revision = 5;
        int64 mod_revision = 6;
        bytes value = 7;
      }
    }

 */

pub struct EtcdClient {
    pub client: awc::Client,
    pub host_addr: String
}

impl EtcdClient {
    pub async fn next_range(&self, seq_name: String, old_value: u64, range_size: u32) -> String {
        let new_value = old_value + range_size;
        let tx = enlarge_tx(seq_name, old_value, new_value);

        make_request(&tx, &self.client, self.host_addr.clone()).await
    }

    pub async fn create_seq(&self, seq_name: String) -> String {
        let tx = create_seq_tx(seq_name);

        make_request(&tx, &self.client, self.host_addr.clone()).await
    }
}


#[derive(Serialize, Deserialize)]
enum CompareResult {
    EQUAL,
    GREATER,
    LESS,
    NotEqual,
}


#[derive(Serialize, Deserialize)]
enum CompareTarget {
    VERSION,
    CREATE,
    MOD,
    VALUE,
}


#[derive(Serialize, Deserialize)]
enum Target {
    #[serde(rename = "version")]
    Version(u64),

    #[serde(rename = "create_revision")]
    CreateRevision(u64),

    #[serde(rename = "mod_revision")]
    ModRevision(u64),

    // base64 encoded
    #[serde(rename = "value")]
    Value(String),
}

#[derive(Serialize, Deserialize)]
struct Comparison {
    pub key: String,
    pub result: CompareResult,

    #[serde(flatten)]
    pub target_value: Target,
    pub target: CompareTarget,
}


#[derive(Serialize, Deserialize)]
struct RequestRange {
    pub key: String,
}

#[derive(Serialize, Deserialize)]
struct RequestPut {
    pub key: String,
    pub value: String,
}


#[derive(Serialize, Deserialize)]
enum Operation {
    #[serde(rename = "requestPut")]
    Put(RequestPut),

    #[serde(rename = "requestRange")]
    Range(RequestRange),
}


#[derive(Serialize, Deserialize)]
struct Transaction {
    pub compare: Vec<Comparison>,
    pub success: Vec<Operation>,
    pub failure: Vec<Operation>,
}


#[test]
pub fn enlarge_test() {
    let enlarge = enlarge_tx("some-sequence-name-2".to_string(), 0, 500);

    println!("{}", serde_json::to_string(&enlarge).unwrap());
}

#[test]
pub fn create_seq_test() {
    let create_seq = create_seq_tx("some-sequence-name-2".to_string());

    println!("{}", serde_json::to_string(&create_seq).unwrap());
}


async fn make_request(tx: &Transaction, client: &awc::Client, host: String) -> String {
    let tx = serde_json::to_string(tx).unwrap();

    // construct request
    let req = client.post(format!("{}/v3/kv/txn", host))
        .insert_header(("User-Agent", "awc/3.0"));

    // send request and await response
    let mut res = req.send_body(tx).await.unwrap();
    let payload = res.body().limit(2000).await.unwrap();

    String::from_utf8(payload.to_vec()).unwrap()
}


fn enlarge_tx(sequence_name: String, old_value: u64, new_value: u64) -> Transaction {
    let key = general_purpose::STANDARD.encode(sequence_name.as_bytes());
    let old_value = general_purpose::STANDARD.encode(old_value.to_be_bytes());
    let new_value = general_purpose::STANDARD.encode(new_value.to_be_bytes());

    Transaction {
        compare: vec![
            Comparison {
                key: key.clone(),
                target_value: Target::Value(old_value),
                target: CompareTarget::VALUE,
                result: CompareResult::EQUAL,
            }],

        success: vec![
            Operation::Put(
                RequestPut { key: key.clone(), value: new_value }
            )
        ],

        failure: vec![
            Operation::Range(
                RequestRange { key }
            )
        ],
    }
}


fn create_seq_tx(sequence_name: String) -> Transaction {
    let key = general_purpose::STANDARD.encode(sequence_name.as_bytes());
    let new_value = general_purpose::STANDARD.encode(0_u64.to_be_bytes());

    Transaction {
        compare: vec![
            // check key version. If it does exist, its version is greater than 0
            Comparison {
                key: key.clone(),
                target_value: Target::Version(0_u64),
                target: CompareTarget::VERSION,
                result: CompareResult::EQUAL,
            }],

        success: vec![
            Operation::Put(
                RequestPut { key: key.clone(), value: new_value }
            )
        ],

        failure: vec![
            Operation::Range(
                RequestRange { key }
            )
        ],
    }
}