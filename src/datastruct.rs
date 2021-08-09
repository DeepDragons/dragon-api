use std::collections::HashMap;

pub const URL: &str = "https://api.zilliqa.com/";

// https://dev.zilliqa.com/docs/apis/api-blockchain-get-current-mini-epoch
// Returns the current TX block number of the network.
// pub const GETMIMIEPOCH: &str = "{\"id\":\"1\",\"jsonrpc\":\"2.0\",\"method\":\"GetCurrentMiniEpoch\",\"params\":[]}";

// https://dev.zilliqa.com/docs/apis/api-contract-get-smartcontract-state/
// Returns the state (mutable) variables of a smart contract address
pub const GETCONTRACTSTATE: &str = "{\"id\":\"1\",\"jsonrpc\":\"2.0\",\"method\":\"GetSmartContractState\",\"params\":[\"b4D83BECB950c096B001a3D1c7aBb10F571ae75f\"]}";
pub const DEFAULT_API_URL: &str = "127.0.0.1:8083";

#[derive(Deserialize, Clone)]
struct Dummy {
    // TODO vec strings
    argtypes: [u8; 0],
    arguments: [u8; 0],
    constructor: String,
}

#[derive(Deserialize, Clone)]
pub struct State {
    _balance: String,
    cloud: String,
    format_img: String,
    max_stage: String,
    migrate_option: Dummy,
    minters: HashMap<String, Dummy>,
    operator_approvals: HashMap<String, String>,
    owned_token_count: HashMap<String, String>,
    token_approvals: HashMap<String, String>,
    pub token_gen_battle: HashMap<String, String>,
    pub token_gen_image: HashMap<String, String>,
    token_id_count: String,
    pub token_owners: HashMap<String, String>,
    pub token_stage: HashMap<String, String>,
    pub token_uris: HashMap<String, String>,
    pub tokens_owner_stage: HashMap<String, HashMap<String, String>>,
    total_supply: String,
}

#[derive(Deserialize)]
pub struct Resp {
    id: String,
    jsonrpc: String,
    pub result: State,
}

#[derive(Deserialize)]
#[serde(default)]
pub struct Page {
    pub limit: usize,
    pub offset: usize,
    pub owner: String,
}
impl Default for Page {
    fn default() -> Self {
        Self {
            limit: 6,
            offset: 0,
            owner: String::new(),
        }
    }
}

#[derive(Serialize, Clone)]
pub struct ShortItem {
    pub id: u64,
    pub url: String,
}

#[derive(Serialize, Clone)]
pub struct Item<'a> {
    pub id: u64,
    pub owner: &'a str,
    pub url: &'a str,
    pub gen_image: &'a str,
    pub gen_fight: &'a str,
    pub stage: u8,
    pub fight_win: u32,
    pub fight_lose: u32,
    pub parents: Vec<ShortItem>,
    pub children: Vec<ShortItem>,
}

#[derive(Serialize)]
pub struct Pagination {
    pub records: usize,
    pub pages: usize,
    pub current_page: usize,
    pub limit: usize,
}

#[derive(Serialize)]
pub struct OkResponse<'a> {
    pub success: bool,
    pub data: Vec<Item<'a>>,
    pub pagination: Pagination,
}

#[derive(Clone)]
pub struct AppState {
    pub id_list: Vec<String>,
    pub owned_id: HashMap<String, Vec<String>>,
    pub contract_state: State,
}
