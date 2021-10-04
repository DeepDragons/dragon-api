use crate::state::*;
use reqwest::StatusCode;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

async fn do_request(body: &'static str) -> String {
    let client = reqwest::Client::new();
    let text: String;
    let mut delay = 0;
    loop {
        sleep(Duration::from_secs(delay)).await;
        delay = (delay + 1) % 15;
        let response = match client.post(URL).body(body).send().await {
            Ok(result) => result,
            Err(_) => continue,
        };
        if response.status() == StatusCode::OK {
            text = match response.text().await {
                Ok(result) => result, // TODO return result and remove text
                Err(_) => continue,
            };
            break;
        }
    }
    // TODO remove debug println and text var...
    if text.len() < 3000 {
        println!("{}", text);
    }
    text
}
pub async fn create() -> AppState {
    let text = do_request(NAMESTATE).await;
    let name_resp: Resp<NameState> = serde_json::from_str(&text).expect("name state");
    let text = do_request(BREEDSTATE).await;
    let breed_resp: Resp<WaitState<BreedItem>> = serde_json::from_str(&text).expect("breed state");
    let mut breed_id_list: Vec<String> = breed_resp.result.waiting_list.keys().cloned().collect();
    let mut breed_owned_id: HashMap<String, Vec<String>> = HashMap::new();
    let mut breed_id_price = HashMap::with_capacity(breed_id_list.len());
    for (id, breed_item) in breed_resp.result.waiting_list {
        breed_id_price.insert(id.clone(), breed_item.arguments[0].clone());
        match breed_owned_id.get_mut(&breed_item.arguments[1]) {
            Some(x) => x.push(id),
            None => {
                breed_owned_id.insert(breed_item.arguments[1].clone(), vec![id]);
            }
        }
    }
    let text = do_request(MARKETSTATE).await;
    let market_resp: Resp<OrderState> = serde_json::from_str(&text).expect("market state");
    let market_len = market_resp.result.orderbook.len();
    let mut market_id_price: HashMap<String, String> = HashMap::with_capacity(market_len);
    let mut market_id_order: HashMap<String, String> = HashMap::with_capacity(market_len);
    let mut market_id_list: Vec<String> = Vec::with_capacity(market_len);
    let mut market_owned_id: HashMap<String, Vec<String>> = HashMap::new();
    // TODO Add with_capacity
    let mut all_id_owner = HashMap::new();
    for i in market_resp.result.orderbook.into_values() {
        let (owner, price, id, order_id) = (
            i.arguments[0].clone(),
            i.arguments[1].clone(),
            i.arguments[2].clone(),
            i.arguments[3].clone(),
        );
        all_id_owner.insert(id.clone(), owner.clone());
        market_id_price.insert(id.clone(), price);
        market_id_order.insert(id.clone(), order_id);
        market_id_list.push(id.clone());
        match market_owned_id.get_mut(&owner) {
            Some(x) => x.push(id),
            None => {
                market_owned_id.insert(owner, vec![id]);
            }
        };
    }
    let text = do_request(MAINSTATE).await;
    // TODO error handling
    let main_resp: Resp<MainState> = serde_json::from_str(&text).expect("main state");
    let text = do_request(BATTLESTATE).await;
    // TODO error handling
    let battle_resp: Resp<WaitState<String>> = serde_json::from_str(&text).expect("battle state");
    drop(text);
    let mut battle_id_list: Vec<String> = battle_resp.result.waiting_list.keys().cloned().collect();
    let mut battle_owned_id: HashMap<String, Vec<String>> = HashMap::new();
    for id in battle_resp.result.waiting_list.keys() {
        // TODO Error handling
        let owner = main_resp.result.token_owners.get(id).unwrap();
        match battle_owned_id.get_mut(owner) {
            Some(x) => x.push(id.to_string()),
            None => {
                battle_owned_id.insert(owner.to_string(), vec![id.to_string()]);
            }
        }
    }

    for (id, owner) in &main_resp.result.token_owners {
        match all_id_owner.get(id) {
            None => {
                all_id_owner.insert(id.to_string(), owner.to_string());
            }
            Some(_) => {}
        }
    }
    // TODO error handling
    let parse_cmp = |a: &String, b: &String| {
        a.parse::<u128>()
            .unwrap_or(u128::MAX)
            .cmp(&b.parse::<u128>().unwrap_or(u128::MAX))
    };
    let mut all_owned_id = HashMap::with_capacity(main_resp.result.tokens_owner_stage.len());
    for (key, val) in &main_resp.result.tokens_owner_stage {
        let mut tokens: Vec<String> = val.keys().cloned().collect();
        if let Some(x) = market_owned_id.get_mut(key) {
            tokens.extend_from_slice(x);
        }
        tokens.sort_unstable_by(parse_cmp);
        all_owned_id.insert(key.to_string(), tokens);
    }
    let all_len = main_resp.result.token_stage.len();
    let mut all_id_rarity: HashMap<String, u8> = HashMap::with_capacity(all_len);
    let mut all_id_strength: HashMap<String, u16> = HashMap::with_capacity(all_len);
    let mut all_id_list: Vec<String> = Vec::with_capacity(all_len);
    for str_id in main_resp.result.token_stage.keys() {
        all_id_list.push(str_id.to_string());
        all_id_rarity.insert(
            str_id.to_string(),
            calc_rarity(
                main_resp
                    .result
                    .token_gen_image
                    .get(str_id)
                    .unwrap_or(&"00000000000000000000000000".to_string()),
            ),
        );
        all_id_strength.insert(
            str_id.to_string(),
            calc_strength(
                main_resp
                    .result
                    .token_gen_battle
                    .get(str_id)
                    .unwrap_or(&"0".to_string()),
            ),
        );
    }
    all_id_list.sort_unstable_by(parse_cmp);
    market_id_list.sort_unstable_by(parse_cmp);
    battle_id_list.sort_unstable_by(parse_cmp);
    breed_id_list.sort_unstable_by(parse_cmp);
    AppState {
        all_id_list,
        all_owned_id,
        all_id_owner,
        all_id_rarity,
        all_id_strength,
        main_state: main_resp.result,
        battle_id_list,
        battle_id_price: battle_resp.result.waiting_list,
        battle_owned_id,
        breed_id_list,
        breed_id_price,
        breed_owned_id,
        market_id_list,
        market_id_price,
        market_id_order,
        market_owned_id,
        id_name: name_resp.result.dragons_name,
    }
}
pub async fn get_block_num() -> u128 {
    loop {
        let text = do_request(GETMIMIEPOCH).await;
        let response: Resp<String> = match serde_json::from_str(&text) {
            Ok(result) => result,
            Err(_) => continue,
        };
        match response.result.parse::<u128>() {
            Ok(result) => return result,
            Err(_) => continue,
        }
    }
}
pub async fn update_state(start_num: u128, app_state: Arc<Mutex<Box<AppState>>>) {
    let mut block_num = start_num;
    let mut delay = 10;
    loop {
        sleep(Duration::from_secs(delay)).await;
        let cur_num = get_block_num().await;
        if cur_num <= block_num {
            if block_num % 100 == 0 {
                delay = 10;
            }
            if delay == 1 {
                delay = 2;
            } else {
                delay = 1;
            }
            continue;
        }
        delay = 25;
        block_num = cur_num;
        let new_state = create().await;
        let mut cur_state = app_state.lock().unwrap();
        *cur_state = Box::new(new_state);
    }
}
// https://github.com/DeepDragons/dragon-zil/blob/master/src/mixins/utils.js#L50
// most of visual gens have 2 parts - type (0-9) and color(0-4)
// head have 1 digit
// claws have 1 digit
// Color Scheme have 3 digits
// MutagenImutable have 3 digits
// e.g. 777 03 03 43 31 14 33 44 11 73 1 4 110 158
//      777 01 64 02 94 03 04 40 24 11 4 1 076 065
// Aura-12   Horns-11   Scales-10   Spots-9   Tail-8   Wings-7
// Spins-6   Body-5   Eyes-4   Head-3   Claws-2   Color Scheme-1   MutagenImutable-0
/* https://github.com/DeepDragons/dragon-zil/blob/master/src/mixins/utils.js#L1
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
fn calc_rarity(gens: &str) -> u8 {
    let gen_to_index = |a, b| gens[a..b].parse::<usize>().unwrap_or(0);
    // https://github.com/DeepDragons/dragon-zil/blob/master/src/mixins/utils.js#L372
    let rarity_sum = RI.aura[gen_to_index(3, 4)]
        + RI.horns[gen_to_index(5, 6)]
        + RI.scales[gen_to_index(7, 8)]
        + RI.spots[gen_to_index(9, 10)]
        + RI.tail[gen_to_index(11, 12)]
        + RI.wings[gen_to_index(13, 14)]
        + RI.body[gen_to_index(17, 18)]
        + RI.eyes[gen_to_index(19, 20)]
        + RI.head[gen_to_index(21, 22)];
    match rarity_sum {
        0..=15 => 0,  //TODO check it? "id":"2490" for example
        16..=23 => 1, // Uncommon
        24..=31 => 2, // Rare
        32..=39 => 3, // Mythical
        40..=47 => 4, // Legendary
        48..=55 => 5, // Imortal
        56..=63 => 6, // Arcana
        _ => 7,       // Ancient
    }
}
// "id":"2851"
// 5271532761388019919425566412768461699999999998899999999999988999999999999996
//  "id":"1894"
//  17213176947417029247062885245301688801479160274101322991103071845030308089925
fn calc_strength(gens: &str) -> u16 {
    let len = gens.len();
    let mut gens_defence: u128 = gens[len - 22..len - 2].parse::<u128>().unwrap_or(0);
    let mut gens_attack: u128 = gens[len - 42..len - 22].parse::<u128>().unwrap_or(0);
    let mut gens_sum = 0;
    for _ in 0..20 {
        gens_sum += (gens_defence % 100) as u16 + (gens_attack % 100) as u16;
        gens_attack /= 100;
        gens_defence /= 100;
    }
    gens_sum
}
