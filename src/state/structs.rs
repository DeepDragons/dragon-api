use std::collections::HashMap;

pub const URL: &str = "https://api.zilliqa.com/";
pub const APPOLO_URL: &str = "https://devex-apollo.zilliqa.com/";

pub const GETFIGHT: &str = "{\"operationName\":\"Fights\",\"variables\":{\"contractAddr\":\"0x21b870dc77921b21f9a98a732786bf812888193c\",\"page\":1,\"perPage\":2147483647},\"query\":\"query Fights($contractAddr: String!, $page: Int, $perPage: Int) {txPagination(page: $page, perPage: $perPage, filter: {OR: [{toAddr: $contractAddr, receipt: {success: true, event_logs: {_eventname: \\\"AfterFightWinLose\\\"}}}]}, sort: TIMESTAMP_ASC) {pageInfo {currentPage perPage pageCount} items {receipt {event_logs { _eventname params {vname value}}}}}}\"}";

// https://dev.zilliqa.com/docs/apis/api-blockchain-get-current-mini-epoch
// Returns the current TX block number of the network.
pub const GETMIMIEPOCH: &str =
    "{\"id\":\"1\",\"jsonrpc\":\"2.0\",\"method\":\"GetCurrentMiniEpoch\",\"params\":[]}";

// https://dev.zilliqa.com/docs/apis/api-contract-get-smartcontract-state/
// Returns the state (mutable) variables of a smart contract address
pub const MAINSTATE: &str = "{\"id\":\"1\",\"jsonrpc\":\"2.0\",\"method\":\"GetSmartContractState\",\"params\":[\"b4D83BECB950c096B001a3D1c7aBb10F571ae75f\"]}";

// https://dev.zilliqa.com/docs/apis/api-contract-get-smartcontract-substate/
// Returns the state (or a part specified) of a smart contract address
pub const BATTLESTATE: &str = "{\"id\":\"1\",\"jsonrpc\":\"2.0\",\"method\":\"GetSmartContractSubState\",\"params\":[\"21B870dc77921B21F9A98a732786Bf812888193c\",\"waiting_list\",[]]}";
pub const BREEDSTATE: &str = "{\"id\":\"1\",\"jsonrpc\":\"2.0\",\"method\":\"GetSmartContractSubState\",\"params\":[\"ade7886ec4a36cb0a7de2f5d18cc7bdae12e3650\",\"waiting_list\",[]]}";
pub const MARKETSTATE: &str = "{\"id\":\"1\",\"jsonrpc\":\"2.0\",\"method\":\"GetSmartContractSubState\",\"params\":[\"7b9b80aaF561Ecd4e89ea55D83d59Ab7aC01A575\",\"orderbook\",[]]}";
pub const NAMESTATE: &str = "{\"id\":\"1\",\"jsonrpc\":\"2.0\",\"method\":\"GetSmartContractSubState\",\"params\":[\"0F5d8f74817E2BC5A09521149094A7860c691D42\",\"dragons_name\",[]]}";

#[derive(Deserialize)]
pub struct EventItem {
    pub vname: String,
    pub value: String,
}

#[derive(Deserialize)]
pub struct EventItems {
    pub _eventname: String,
    pub params: Vec<EventItem>,
}

#[derive(Deserialize)]
pub struct EventLogs {
    pub event_logs: Vec<EventItems>,
}

#[derive(Deserialize)]
pub struct Receipt {
    pub receipt: EventLogs,
}

#[derive(Deserialize)]
pub struct PageInfo {
    pub currentPage: u64,
    pub perPage: u64,
    pub pageCount: u64,
}

#[derive(Deserialize)]
pub struct TxPaginationItem {
    pub pageInfo: PageInfo,
    pub items: Vec<Receipt>,
}

#[derive(Deserialize)]
pub struct TxPagination {
    pub txPagination: TxPaginationItem,
}

#[derive(Deserialize)]
pub struct Data {
    pub data: TxPagination,
}

#[derive(Deserialize, Clone)]
struct Dummy {
    // TODO vec strings
    argtypes: [u8; 0],
    arguments: [u8; 0],
    constructor: String,
}

// https://github.com/DeepDragons/DragonZILContracts/blob/main/MarketPlace.scilla#L209
#[derive(Deserialize)]
pub struct MarketItem {
    argtypes: [u8; 0],
    // Order of ByStr20 Uint128 Uint256 Uint256 (owner, price, id, order_id)
    pub arguments: [String; 4],
    constructor: String,
}

//https://github.com/DeepDragons/DragonZILContracts/blob/main/MarketPlace.scilla#L90
#[derive(Deserialize)]
pub struct OrderState {
    // Map Uint256 Order (order_id -> Order)
    pub orderbook: HashMap<String, MarketItem>,
}

// https://github.com/DeepDragons/DragonZILContracts/blob/main/BreedPlace.scilla#L256
// waiting_list: Map Uint256 (Pair Uint128 ByStr20) (id -> (price, owner))
#[derive(Deserialize, Clone)]
pub struct BreedItem {
    argtypes: [String; 2],
    pub arguments: [String; 2], // ar..s[0] is price, ar..s[1] is owner
    constructor: String,
}

#[derive(Deserialize)]
pub struct WaitState<T> {
    pub waiting_list: HashMap<String, T>,
}

#[derive(Deserialize)]
pub struct NameState {
    pub dragons_name: HashMap<String, String>,
}

#[derive(Deserialize)]
pub struct Resp<T> {
    id: String,
    jsonrpc: String,
    pub result: T,
}

type HMStrings = HashMap<String, String>;

// https://github.com/DeepDragons/DragonZILContracts/blob/main/DragonZIL.scilla#L150
#[derive(Deserialize, Clone)]
pub struct MainState {
    _balance: String,
    cloud: String,
    format_img: String,
    max_stage: String,
    migrate_option: Dummy,
    minters: HashMap<String, Dummy>,
    operator_approvals: HashMap<String, HashMap<String, Dummy>>,
    // Map ByStr20 Uint25 (owner -> count)
    owned_token_count: HMStrings,
    token_approvals: HMStrings,
    // Map Uint256 Uint256 (id -> gens)
    pub token_gen_battle: HMStrings,
    // Map Uint256 Uint256 (id -> gens)
    pub token_gen_image: HMStrings,
    token_id_count: String,
    //Map Uint256 ByStr20 (id -> owner)
    pub token_owners: HMStrings,
    // Map Uint256 Uint32 (id -> stage)
    pub token_stage: HMStrings,
    // Map Uint256 String (id -> uri)
    pub token_uris: HMStrings,
    // Map ByStr20 (Map Uint256 Uint32) (owner -> (id -> stage))
    pub tokens_owner_stage: HashMap<String, HMStrings>,
    total_supply: String,
}

type HMVecStrings = HashMap<String, Vec<String>>;

#[derive(Clone)]
pub struct AppState {
    pub all_id_list: Vec<String>,
    pub all_owned_id: HMVecStrings, //             (owner -> Vec<id>)
    pub all_id_owner: HMStrings,    //                (id -> Owner)
    pub all_id_rarity: HashMap<String, u8>, //     (id -> rarity)
    pub all_id_strength: HashMap<String, u16>, // (id -> strength)
    pub all_id_fights: HashMap<String, (u32, u32)>, // (id -> (win, lose))
    pub main_state: MainState,
    pub battle_id_list: Vec<String>,
    pub battle_id_price: HMStrings,    //             (id -> price)
    pub battle_owned_id: HMVecStrings, //          (owner -> Vec<id>)
    pub breed_id_list: Vec<String>,
    pub breed_id_price: HMStrings,    //              (id -> price)
    pub breed_owned_id: HMVecStrings, //           (owner -> Vec<id>)
    pub market_id_list: Vec<String>,
    pub market_id_price: HMStrings,    //             (id -> price)
    pub market_id_order: HMStrings,    //             (id -> order_id)
    pub market_owned_id: HMVecStrings, //          (owner -> Vec<id>)
    pub id_name: HMStrings,            //            (id -> name)
}

/*
 * https://github.com/DeepDragons/dragon-zil/blob/master/src/mixins/utils.js
 * None      0
 * Common    1
 * Uncommon  2
 * Rare      3
 * Mythical  4
 * Legendary 5
 * Immortal  6
 * Arcana    7
 * Ancient   8
 */
pub const RI: RarityConst = RarityConst {
    aura: [0, 2, 3, 3, 4, 5],
    horns: [0, 2, 3, 3, 3, 3, 4, 5],
    scales: [1, 2, 3, 4, 5],
    spots: [0, 1, 2, 2, 2, 2, 2, 2, 8, 5],
    tail: [0, 2, 3, 3, 3, 3, 3, 4, 5],
    wings: [0, 1, 2, 3, 4, 5],
    body: [0, 1, 4, 6],
    eyes: [0, 1, 3, 3, 3, 3, 4, 4, 5, 6],
    head: [0, 1, 3, 5, 6, 7],
};

pub struct RarityConst {
    pub aura: [u8; 6],
    pub horns: [u8; 8],
    pub scales: [u8; 5],
    pub spots: [u8; 10],
    pub tail: [u8; 9],
    pub wings: [u8; 6],
    pub body: [u8; 4],
    pub eyes: [u8; 10],
    pub head: [u8; 6],
}
