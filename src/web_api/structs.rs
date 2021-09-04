#[derive(Deserialize)]
#[serde(default)]
pub struct Page {
    pub limit: usize,
    pub offset: usize,
    pub owner: String,
    pub stage: u8,
    pub sort: u8, // 0 and _ - id, 1 - rarity, 2 - strong, 3 - price,
    pub start_price: u64,
    pub end_price: u64,
}
impl Default for Page {
    fn default() -> Self {
        Self {
            limit: 6,
            offset: 0,
            owner: String::new(),
            stage: 255,
            sort: 0,
            start_price: 0,
            end_price: u64::MAX,
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
    pub id: &'a str,
    pub owner: &'a str,
    pub url: &'a str,
    pub gen_image: &'a str,
    pub gen_fight: &'a str,
    pub stage: u8,
    pub rarity: u8,
    pub fight_win: u32,
    pub fight_lose: u32,
    pub actions: Vec<(u8, &'a str)>,
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
