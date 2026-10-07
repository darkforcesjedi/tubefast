use crate::auth;
use serde::de::{DeserializeSeed, Deserializer, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashSet;
use std::io::BufReader;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const MUSIC_API: &str = "https://music.youtube.com/youtubei/v1/";
const PLAYER_API: &str = "https://www.youtube.com/youtubei/v1/player?prettyPrint=false";
const VISITOR_API: &str = "https://www.youtube.com/youtubei/v1/visitor_id?prettyPrint=false";
const LANGUAGE: &str = "en";
const WEB_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36";
const REMIX_VERSION: &str = "1.20260707.12.00";
const REMIX_CLIENT_ID: &str = "67";
const PLAYER_CLIENT: &str = "VISIONOS";
const PLAYER_VERSION: &str = "1.02";
const PLAYER_CLIENT_ID: &str = "101";
const PLAYER_DEVICE: &str = "RealityDevice17,1";
const PLAYER_OS: &str = "26.5.23O471";
pub const PLAYER_UA: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 15_7_3) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.0 Safari/605.1.15";
const WATCH_API: &str = "https://www.youtube.com/youtubei/v1/next?prettyPrint=false";
const MWEB_VERSION: &str = "2.20260708.05.00";
const MWEB_CLIENT_ID: &str = "2";
const MWEB_UA: &str = "Mozilla/5.0 (iPad; CPU OS 16_7_10 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.6 Mobile/15E148 Safari/604.1,gzip(gfe)";
const SONG_CARD_KEY: &str = "\"videoAttributeViewModel\"";
const COVER_HOST: &str = "googleusercontent.com";
const THUMB_WIDTH: u64 = 700;
const COVER_SIZE: &str = "=w544-h544-l90-rj";
pub const VIDEO_FRAME_HOST: &str = "i.ytimg.com";
const AAC_ITAG: u64 = 140;
const RELATED_ITEMS: usize = 18;
const EXPLORE_ORDER: usize = 100;
const HOME_ORDER: usize = 200;
const PERSONAL_PAGES: usize = 3;
const SESSION_REJECTED: &str = "YouTube did not accept this sign-in. Sign in again.";
const UNUSED_KEYS: [&str; 13] = [
    "menu",
    "thumbnailOverlay",
    "trackingParams",
    "clickTrackingParams",
    "loggingContext",
    "multiSelectCheckbox",
    "accessibility",
    "accessibilityData",
    "frameworkUpdates",
    "serviceTrackingParams",
    "buttons",
    "badges",
    "playerOverlays",
];

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub title: String,
    pub subtitle: String,
    pub extra: String,
    pub duration: String,
    pub thumb: String,
    pub video_id: String,
    pub browse_id: String,
    pub artist_id: String,
    pub album_id: String,
    pub is_artist: bool,
}

impl Item {
    pub fn is_song(&self) -> bool {
        !self.video_id.is_empty()
    }

    pub fn key(&self) -> &str {
        if self.is_song() { &self.video_id } else { &self.browse_id }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Layout {
    Cards,
    Compact,
    Tracks,
    Top,
}

#[derive(Clone, Debug)]
pub struct Section {
    pub title: String,
    pub layout: Layout,
    pub items: Vec<Item>,
}

#[derive(Clone, Debug, Default)]
pub struct Page {
    pub header: Option<Item>,
    pub sections: Vec<Section>,
    pub partial: bool,
}

impl Page {
    pub fn songs(&self) -> Vec<Item> {
        self.sections
            .iter()
            .flat_map(|s| &s.items)
            .filter(|i| i.is_song())
            .cloned()
            .collect()
    }
}

pub struct Stream {
    pub url: String,
    pub len: u64,
    pub duration_ms: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Account {
    pub name: String,
    pub photo: String,
}

pub struct Client {
    pub agent: ureq::Agent,
    visitor: Mutex<String>,
    session: Mutex<Option<String>>,
    expired: AtomicBool,
}

impl Client {
    pub fn new(visitor: String) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(10))
            .timeout_read(Duration::from_secs(30))
            .user_agent(WEB_UA)
            .build();
        Self {
            agent,
            visitor: Mutex::new(visitor),
            session: Mutex::new(None),
            expired: AtomicBool::new(false),
        }
    }

    pub fn visitor(&self) -> String {
        self.visitor.lock().unwrap().clone()
    }

    pub fn set_session(&self, cookie: Option<String>) {
        *self.session.lock().unwrap() = cookie;
    }

    pub fn signed_in(&self) -> bool {
        self.session.lock().unwrap().is_some()
    }

    pub fn take_expired(&self) -> bool {
        self.expired.swap(false, Relaxed)
    }

    pub fn sign_in(&self, cookie: String) -> Result<Account, String> {
        self.set_session(Some(cookie));
        let account = self.music("account/account_menu", json!({}), &[]).and_then(|response| {
            let header = find(&response, "activeAccountHeaderRenderer").ok_or(SESSION_REJECTED)?;
            Ok(Account {
                name: text(&header["accountName"]),
                photo: thumb(&header["accountPhoto"]),
            })
        });
        if account.is_err() {
            self.set_session(None);
            self.expired.store(false, Relaxed);
        }
        account
    }

    pub fn cover(&self, video_id: &str) -> Result<String, String> {
        let body = json!({
            "context": { "client": { "clientName": "MWEB", "clientVersion": MWEB_VERSION, "hl": LANGUAGE } },
            "videoId": video_id,
        });
        let request = self
            .agent
            .post(WATCH_API)
            .set("User-Agent", MWEB_UA)
            .set("X-YouTube-Client-Name", MWEB_CLIENT_ID)
            .set("X-YouTube-Client-Version", MWEB_VERSION);
        let watch_page = request
            .send_json(body)
            .map_err(describe)?
            .into_string()
            .map_err(|e| e.to_string())?;
        Ok(single_song_cover(&watch_page))
    }

    pub fn library(&self) -> Result<Vec<Item>, String> {
        let page = self.browse("FEmusic_liked_playlists")?;
        Ok(page
            .sections
            .into_iter()
            .flat_map(|section| section.items)
            .filter(|item| !item.is_song())
            .collect())
    }

    fn feed(&self, order: usize, pages: usize, emit: &(dyn Fn(usize, Vec<Section>) + Sync)) -> Result<(), String> {
        let mut response = self.music("browse", json!({ "browseId": "FEmusic_home" }), &[])?;
        let mut seen = HashSet::new();
        for step in 0..pages {
            let mut sections = parse_page(&response).sections;
            sections.retain(|section| seen.insert(section.title.clone()));
            emit(order + step, sections);
            let Some(token) = continuation(&response).filter(|_| step + 1 < pages) else {
                break;
            };
            let params = [("ctoken", token.as_str()), ("continuation", token.as_str()), ("type", "next")];
            response = self.music("browse", json!({ "continuation": token }), &params)?;
        }
        Ok(())
    }

    pub fn home(&self, seeds: &[Item], emit: &(dyn Fn(usize, Vec<Section>) + Sync)) -> Result<(), String> {
        let (explore_order, home_order, pages) = if self.signed_in() {
            (HOME_ORDER, EXPLORE_ORDER, PERSONAL_PAGES)
        } else {
            (EXPLORE_ORDER, HOME_ORDER, 1)
        };
        std::thread::scope(|scope| {
            let related: Vec<_> = seeds
                .iter()
                .enumerate()
                .map(|(order, seed)| {
                    scope.spawn(move || {
                        let radio = self.radio(&seed.video_id)?;
                        let items: Vec<Item> = radio
                            .into_iter()
                            .filter(|item| item.video_id != seed.video_id)
                            .take(RELATED_ITEMS)
                            .collect();
                        if !items.is_empty() {
                            emit(
                                order,
                                vec![Section {
                                    title: format!("More like {}", seed.title),
                                    layout: Layout::Compact,
                                    items,
                                }],
                            );
                        }
                        Ok(())
                    })
                })
                .collect();
            let explore = scope.spawn(move || self.browse("FEmusic_explore").map(|page| emit(explore_order, page.sections)));
            let home = self.feed(home_order, pages, emit);
            let finished = related.into_iter().chain([explore]).map(|task| task.join().unwrap()).chain([home]);
            let mut outcome = Err(String::new());
            for result in finished {
                if outcome.is_err() {
                    outcome = result;
                }
            }
            outcome
        })
    }

    pub fn browse(&self, browse_id: &str) -> Result<Page, String> {
        let response = self.music("browse", json!({ "browseId": browse_id }), &[])?;
        let mut page = parse_page(&response);
        if let Some(header) = &mut page.header {
            header.browse_id = browse_id.to_owned();
            header.is_artist = browse_id.starts_with("UC");
            let (thumb, artist) = (header.thumb.clone(), header.extra.clone());
            for item in page.sections.iter_mut().flat_map(|s| &mut s.items) {
                if item.thumb.is_empty() {
                    item.thumb = thumb.clone();
                }
                if item.subtitle.is_empty() {
                    item.subtitle = artist.clone();
                }
            }
        }
        Ok(page)
    }

    pub fn search(&self, query: &str) -> Result<Page, String> {
        let response = self.music("search", json!({ "query": query }), &[])?;
        let mut page = parse_page(&response);
        page.header = None;
        for section in page.sections.iter_mut().filter(|section| section.title.is_empty()) {
            section.title = "Results".to_owned();
        }
        Ok(page)
    }

    pub fn radio(&self, video_id: &str) -> Result<Vec<Item>, String> {
        let body = json!({
            "videoId": video_id,
            "playlistId": format!("RDAMVM{video_id}"),
            "isAudioOnly": true,
        });
        Ok(parse_page(&self.music("next", body, &[])?).songs())
    }

    pub fn stream(&self, video_id: &str) -> Result<Stream, String> {
        let first = self.player(video_id)?;
        let response = if playable(&first) {
            first
        } else {
            self.visitor.lock().unwrap().clear();
            self.player(video_id)?
        };
        if !playable(&response) {
            let status = &response["playabilityStatus"];
            let reason = status["reason"].as_str().or(status["status"].as_str());
            return Err(reason.unwrap_or("This track cannot be played").to_owned());
        }
        let formats = response["streamingData"]["adaptiveFormats"].as_array();
        let format = formats
            .and_then(|all| all.iter().find(|f| f["itag"].as_u64() == Some(AAC_ITAG) && f["url"].is_string()))
            .ok_or("No compatible audio stream for this track")?;
        let number = |key: &str| format[key].as_str().and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
        Ok(Stream {
            url: format["url"].as_str().unwrap_or_default().to_owned(),
            len: number("contentLength"),
            duration_ms: number("approxDurationMs"),
        })
    }

    fn player(&self, video_id: &str) -> Result<Value, String> {
        let visitor = self.ensure_visitor()?;
        let body = json!({
            "context": { "client": {
                "clientName": PLAYER_CLIENT,
                "clientVersion": PLAYER_VERSION,
                "deviceMake": "Apple",
                "deviceModel": PLAYER_DEVICE,
                "osName": "visionOS",
                "osVersion": PLAYER_OS,
                "hl": LANGUAGE,
                "visitorData": visitor,
            }},
            "videoId": video_id,
            "contentCheckOk": true,
            "racyCheckOk": true,
        });
        let request = self
            .agent
            .post(PLAYER_API)
            .set("User-Agent", PLAYER_UA)
            .set("X-YouTube-Client-Name", PLAYER_CLIENT_ID)
            .set("X-YouTube-Client-Version", PLAYER_VERSION)
            .set("X-Goog-Visitor-Id", &visitor);
        send(request, body)
    }

    fn ensure_visitor(&self) -> Result<String, String> {
        let known = self.visitor();
        if !known.is_empty() {
            return Ok(known);
        }
        let body = json!({ "context": { "client": { "clientName": "WEB", "clientVersion": "2.20260707.00.00", "hl": LANGUAGE } } });
        let response = send(self.agent.post(VISITOR_API), body)?;
        let visitor = response["responseContext"]["visitorData"]
            .as_str()
            .ok_or("YouTube did not issue a visitor id")?
            .to_owned();
        *self.visitor.lock().unwrap() = visitor.clone();
        Ok(visitor)
    }

    fn music(&self, endpoint: &str, mut body: Value, params: &[(&str, &str)]) -> Result<Value, String> {
        body["context"] = json!({ "client": { "clientName": "WEB_REMIX", "clientVersion": REMIX_VERSION, "hl": LANGUAGE } });
        let mut request = self
            .agent
            .post(&format!("{MUSIC_API}{endpoint}?prettyPrint=false"))
            .set("Origin", auth::ORIGIN)
            .set("X-YouTube-Client-Name", REMIX_CLIENT_ID)
            .set("X-YouTube-Client-Version", REMIX_VERSION);
        for (name, value) in params {
            request = request.query(name, value);
        }
        let cookie = self.session.lock().unwrap().clone();
        if let Some(cookie) = &cookie {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |elapsed| elapsed.as_secs());
            let signature = auth::authorization(cookie, now).unwrap_or_default();
            request = request
                .set("Cookie", cookie)
                .set("Authorization", &signature)
                .set("X-Origin", auth::ORIGIN)
                .set("X-Goog-AuthUser", "0");
        }
        let response = match request.send_json(body) {
            Ok(response) => read(response)?,
            Err(ureq::Error::Status(401, _)) if cookie.is_some() => {
                self.set_session(None);
                self.expired.store(true, Relaxed);
                return Err(SESSION_REJECTED.to_owned());
            }
            Err(error) => return Err(describe(error)),
        };
        if let Some(visitor) = response["responseContext"]["visitorData"].as_str() {
            let mut known = self.visitor.lock().unwrap();
            if known.is_empty() {
                *known = visitor.to_owned();
            }
        }
        Ok(response)
    }
}

fn send(request: ureq::Request, body: Value) -> Result<Value, String> {
    read(request.send_json(body).map_err(describe)?)
}

fn read(response: ureq::Response) -> Result<Value, String> {
    let reader = BufReader::new(response.into_reader());
    Lean.deserialize(&mut serde_json::Deserializer::from_reader(reader))
        .map_err(|e| e.to_string())
}

fn find<'a>(node: &'a Value, key: &str) -> Option<&'a Value> {
    match node {
        Value::Array(list) => list.iter().find_map(|child| find(child, key)),
        Value::Object(map) => map.get(key).or_else(|| map.values().find_map(|child| find(child, key))),
        _ => None,
    }
}

fn single_song_cover(watch_page: &str) -> String {
    let mut mentions = watch_page
        .match_indices(SONG_CARD_KEY)
        .map(|(at, key)| &watch_page[at + key.len()..]);
    let song = match (mentions.next(), mentions.next()) {
        (Some(only), None) => only.find('{').map(|open| &only[open..]),
        _ => None,
    };
    let song: Value = song
        .and_then(|json| serde_json::Deserializer::from_str(json).into_iter().next()?.ok())
        .unwrap_or_default();
    let image = song.pointer("/image/sources/0/url").and_then(Value::as_str).unwrap_or_default();
    match (image.contains(COVER_HOST), image.contains('=')) {
        (false, _) => String::new(),
        (true, true) => image.to_owned(),
        (true, false) => format!("{image}{COVER_SIZE}"),
    }
}

fn continuation(response: &Value) -> Option<String> {
    let classic = find(response, "nextContinuationData").map(|data| &data["continuation"]);
    let token = classic.or_else(|| find(response, "continuationCommand").map(|command| &command["token"]))?;
    token.as_str().map(str::to_owned)
}

struct Lean;

impl<'de> DeserializeSeed<'de> for Lean {
    type Value = Value;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for Lean {
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a JSON value")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Value, E> {
        Ok(value.into())
    }

    fn visit_u64<E>(self, value: u64) -> Result<Value, E> {
        Ok(value.into())
    }

    fn visit_f64<E>(self, value: f64) -> Result<Value, E> {
        Ok(value.into())
    }

    fn visit_str<E>(self, value: &str) -> Result<Value, E> {
        Ok(Value::String(value.to_owned()))
    }

    fn visit_unit<E>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
        let mut list = Vec::new();
        while let Some(value) = seq.next_element_seed(Lean)? {
            list.push(value);
        }
        Ok(Value::Array(list))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        let mut object = serde_json::Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if UNUSED_KEYS.contains(&key.as_str()) {
                map.next_value::<IgnoredAny>()?;
            } else {
                object.insert(key, map.next_value_seed(Lean)?);
            }
        }
        Ok(Value::Object(object))
    }
}

fn playable(response: &Value) -> bool {
    response["playabilityStatus"]["status"] == "OK"
}

fn describe(error: ureq::Error) -> String {
    match error {
        ureq::Error::Status(code, _) => format!("YouTube answered with HTTP {code}"),
        ureq::Error::Transport(_) => "Could not reach YouTube. Check your connection.".to_owned(),
    }
}

pub fn sized(url: &str, px: u32) -> String {
    let Some(at) = url.rfind('=') else { return url.to_owned() };
    let (base, params) = url.split_at(at);
    if params.starts_with("=w") {
        let rest = params.splitn(3, '-').nth(2).map(|r| format!("-{r}")).unwrap_or_default();
        format!("{base}=w{px}-h{px}{rest}")
    } else if params.starts_with("=s") {
        format!("{base}=s{px}")
    } else {
        url.to_owned()
    }
}

pub fn parse_page(response: &Value) -> Page {
    let mut page = Page::default();
    collect(response, &mut page);
    page.sections.retain(|s| !s.items.is_empty());
    page
}

const SHELVES: [&str; 5] = [
    "musicCarouselShelfRenderer",
    "musicShelfRenderer",
    "musicPlaylistShelfRenderer",
    "musicCardShelfRenderer",
    "gridRenderer",
];
const ITEMS: [&str; 3] = [
    "musicResponsiveListItemRenderer",
    "musicTwoRowItemRenderer",
    "playlistPanelVideoRenderer",
];
const HEADERS: [&str; 4] = [
    "musicResponsiveHeaderRenderer",
    "musicImmersiveHeaderRenderer",
    "musicVisualHeaderRenderer",
    "musicDetailHeaderRenderer",
];

fn collect(node: &Value, page: &mut Page) {
    match node {
        Value::Array(list) => list.iter().for_each(|child| collect(child, page)),
        Value::Object(map) => {
            for (key, child) in map {
                let key = key.as_str();
                if SHELVES.contains(&key) {
                    page.sections.push(shelf(key, child));
                } else if ITEMS.contains(&key) {
                    let loose = matches!(page.sections.last(), Some(s) if s.title.is_empty() && s.layout == Layout::Tracks);
                    if !loose {
                        page.sections.push(Section {
                            title: String::new(),
                            layout: Layout::Tracks,
                            items: Vec::new(),
                        });
                    }
                    page.sections.last_mut().unwrap().items.extend(item(child));
                } else if HEADERS.contains(&key) {
                    page.header = Some(header(child));
                } else {
                    collect(child, page);
                }
            }
        }
        _ => {}
    }
}

fn shelf(kind: &str, node: &Value) -> Section {
    let children = node["contents"].as_array().or(node["items"].as_array());
    let entries = || children.into_iter().flatten().filter_map(Value::as_object).flat_map(|o| o.iter());
    let mut items: Vec<Item> = entries()
        .filter(|(k, _)| ITEMS.contains(&k.as_str()))
        .filter_map(|(_, v)| item(v))
        .collect();
    let cards = entries().next().is_some_and(|(k, _)| k == "musicTwoRowItemRenderer");
    let (title, layout) = match kind {
        "musicCardShelfRenderer" => {
            let top = card(node);
            for item in items.iter_mut().filter(|item| top.is_artist && item.subtitle.is_empty()) {
                item.subtitle = top.title.clone();
            }
            items.insert(0, top);
            ("Top result".to_owned(), Layout::Top)
        }
        "musicCarouselShelfRenderer" | "gridRenderer" => {
            let basic = &node["header"]["musicCarouselShelfBasicHeaderRenderer"]["title"];
            let grid = &node["header"]["gridHeaderRenderer"]["title"];
            (text(basic) + &text(grid), if cards { Layout::Cards } else { Layout::Compact })
        }
        _ => (text(&node["title"]), Layout::Tracks),
    };
    Section { title, layout, items }
}

fn card(node: &Value) -> Item {
    let mut it = Item {
        title: text(&node["title"]),
        subtitle: text(&node["subtitle"]),
        thumb: thumb(&node["thumbnail"]),
        video_id: first_str(
            node,
            &[
                "/onTap/watchEndpoint/videoId",
                "/title/runs/0/navigationEndpoint/watchEndpoint/videoId",
            ],
        ),
        browse_id: first_str(
            node,
            &[
                "/onTap/browseEndpoint/browseId",
                "/title/runs/0/navigationEndpoint/browseEndpoint/browseId",
            ],
        ),
        ..Item::default()
    };
    it.is_artist = !it.is_song() && it.browse_id.starts_with("UC");
    tidy(&mut it);
    it
}

fn tidy(it: &mut Item) {
    let mut parts: Vec<&str> = it.subtitle.split(" • ").collect();
    if parts.first() == Some(&"Song") {
        parts.remove(0);
    }
    let timed = parts
        .last()
        .is_some_and(|last| last.contains(':') && last.chars().all(|c| c.is_ascii_digit() || c == ':'));
    if timed && it.duration.is_empty() {
        it.duration = parts.pop().unwrap_or_default().to_owned();
    }
    it.subtitle = parts.join(" • ");
}

fn header(node: &Value) -> Item {
    let details = [
        text(&node["subtitle"]),
        text(&node["secondSubtitle"]),
        text(&node["monthlyListenerCount"]),
    ];
    Item {
        title: text(&node["title"]),
        subtitle: details.into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" • "),
        extra: text(&node["straplineTextOne"]),
        thumb: thumb(&node["thumbnail"]),
        ..Item::default()
    }
}

fn item(node: &Value) -> Option<Item> {
    let column = |index: usize| text(&node["flexColumns"][index]["musicResponsiveListItemFlexColumnRenderer"]["text"]);
    let title = [text(&node["title"]), column(0)].into_iter().find(|s| !s.is_empty())?;
    let bylines = [
        text(&node["subtitle"]),
        text(&node["shortBylineText"]),
        text(&node["longBylineText"]),
        column(1),
    ];
    let subtitle = bylines.into_iter().find(|s| !s.is_empty());
    let fixed = text(&node["fixedColumns"][0]["musicResponsiveListItemFixedColumnRenderer"]["text"]);
    let mut it = Item {
        title,
        subtitle: subtitle.unwrap_or_default(),
        extra: [column(2), column(3)]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" • "),
        duration: fixed + &text(&node["lengthText"]),
        thumb: thumb(node),
        video_id: first_str(
            node,
            &[
                "/playlistItemData/videoId",
                "/videoId",
                "/navigationEndpoint/watchEndpoint/videoId",
                "/flexColumns/0/musicResponsiveListItemFlexColumnRenderer/text/runs/0/navigationEndpoint/watchEndpoint/videoId",
                "/overlay/musicItemThumbnailOverlayRenderer/content/musicPlayButtonRenderer/playNavigationEndpoint/watchEndpoint/videoId",
            ],
        ),
        browse_id: first_str(
            node,
            &[
                "/navigationEndpoint/browseEndpoint/browseId",
                "/title/runs/0/navigationEndpoint/browseEndpoint/browseId",
                "/flexColumns/0/musicResponsiveListItemFlexColumnRenderer/text/runs/0/navigationEndpoint/browseEndpoint/browseId",
            ],
        ),
        ..Item::default()
    };
    links(node, &mut it);
    it.is_artist = !it.is_song() && it.browse_id.starts_with("UC");
    tidy(&mut it);
    (it.is_song() || !it.browse_id.is_empty()).then_some(it)
}

fn links(node: &Value, it: &mut Item) {
    match node {
        Value::Array(list) => list.iter().for_each(|child| links(child, it)),
        Value::Object(map) => {
            for (key, child) in map {
                if key == "browseEndpoint" {
                    let kind = child.pointer("/browseEndpointContextSupportedConfigs/browseEndpointContextMusicConfig/pageType");
                    let id = child["browseId"].as_str().unwrap_or_default();
                    match kind.and_then(Value::as_str) {
                        Some("MUSIC_PAGE_TYPE_ARTIST") if it.artist_id.is_empty() => it.artist_id = id.to_owned(),
                        Some("MUSIC_PAGE_TYPE_ALBUM") if it.album_id.is_empty() => it.album_id = id.to_owned(),
                        _ => {}
                    }
                } else if key != "menu" {
                    links(child, it);
                }
            }
        }
        _ => {}
    }
}

fn text(node: &Value) -> String {
    node["runs"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|run| run["text"].as_str())
        .collect()
}

fn first_str(node: &Value, pointers: &[&str]) -> String {
    pointers
        .iter()
        .find_map(|p| node.pointer(p)?.as_str())
        .unwrap_or_default()
        .to_owned()
}

fn thumb(node: &Value) -> String {
    match node {
        Value::Array(list) => list.iter().map(thumb).find(|url| !url.is_empty()).unwrap_or_default(),
        Value::Object(map) => {
            let sizes = map.get("thumbnails").and_then(Value::as_array);
            let enough = |size: &&Value| size["width"].as_u64().is_some_and(|width| width >= THUMB_WIDTH);
            if let Some(url) = sizes
                .and_then(|sizes| sizes.iter().find(enough).or(sizes.last()))
                .and_then(|size| size["url"].as_str())
            {
                return url.to_owned();
            }
            map.iter()
                .filter(|(key, _)| *key != "menu")
                .map(|(_, child)| thumb(child))
                .find(|url| !url.is_empty())
                .unwrap_or_default()
        }
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resizes_thumbnails_without_touching_other_hosts() {
        assert_eq!(
            sized("https://lh3.googleusercontent.com/abc=w60-h60-l90-rj", 240),
            "https://lh3.googleusercontent.com/abc=w240-h240-l90-rj"
        );
        assert_eq!(
            sized("https://yt3.googleusercontent.com/abc=s192", 512),
            "https://yt3.googleusercontent.com/abc=s512"
        );
        assert_eq!(
            sized("https://i.ytimg.com/vi/x/hq720.jpg?sqp=a&rs=b", 240),
            "https://i.ytimg.com/vi/x/hq720.jpg?sqp=a&rs=b"
        );
    }

    #[test]
    fn takes_a_cover_only_when_youtube_names_exactly_one_song() {
        let song = |url: &str| json!({ "videoAttributeViewModel": { "title": "Loser", "image": { "sources": [{ "url": url }] } } });
        let art = "https://yt3.googleusercontent.com/abc";
        assert_eq!(
            single_song_cover(&json!({ "cards": [song(art)] }).to_string()),
            "https://yt3.googleusercontent.com/abc=w544-h544-l90-rj"
        );
        assert_eq!(
            single_song_cover(&json!({ "cards": [song("https://lh3.googleusercontent.com/abc=w60-h60")] }).to_string()),
            "https://lh3.googleusercontent.com/abc=w60-h60"
        );
        assert_eq!(single_song_cover(&json!({ "cards": [song(art), song(art)] }).to_string()), "");
        assert_eq!(
            single_song_cover(&json!({ "cards": [song("https://www.youtube.com/img/watch/yt_music_channel.jpeg")] }).to_string()),
            ""
        );
        assert_eq!(single_song_cover("{\"cards\":[]}"), "");
    }

    #[test]
    fn parses_shelves_loose_results_and_headers() {
        let song = json!({ "musicResponsiveListItemRenderer": {
            "playlistItemData": { "videoId": "abcdefghijk" },
            "thumbnail": { "musicThumbnailRenderer": { "thumbnail": { "thumbnails": [{ "url": "small" }, { "url": "large" }] } } },
            "menu": { "items": [{ "browseEndpoint": { "browseId": "UCmenu", "browseEndpointContextSupportedConfigs": { "browseEndpointContextMusicConfig": { "pageType": "MUSIC_PAGE_TYPE_ARTIST" } } } }] },
            "flexColumns": [
                { "musicResponsiveListItemFlexColumnRenderer": { "text": { "runs": [{ "text": "Kuzu Kuzu" }] } } },
                { "musicResponsiveListItemFlexColumnRenderer": { "text": { "runs": [
                    { "text": "Song" }, { "text": " • " },
                    { "text": "Tarkan", "navigationEndpoint": { "browseEndpoint": { "browseId": "UCartist", "browseEndpointContextSupportedConfigs": { "browseEndpointContextMusicConfig": { "pageType": "MUSIC_PAGE_TYPE_ARTIST" } } } } },
                    { "text": " • " }, { "text": "3:52" }
                ] } } }
            ]
        }});
        let album = json!({ "musicTwoRowItemRenderer": {
            "title": { "runs": [{ "text": "Karma" }] },
            "subtitle": { "runs": [{ "text": "Album" }] },
            "navigationEndpoint": { "browseEndpoint": { "browseId": "MPREb_album" } }
        }});
        let response = json!({ "contents": [
            { "musicResponsiveHeaderRenderer": { "title": { "runs": [{ "text": "Karma" }] }, "straplineTextOne": { "runs": [{ "text": "Tarkan" }] } } },
            { "musicCarouselShelfRenderer": { "header": { "musicCarouselShelfBasicHeaderRenderer": { "title": { "runs": [{ "text": "Albums" }] } } }, "contents": [album] } },
            { "itemSectionRenderer": { "contents": [song.clone()] } },
            { "itemSectionRenderer": { "contents": [song] } }
        ]});
        let page = parse_page(&response);
        assert_eq!(page.header.as_ref().unwrap().extra, "Tarkan");
        assert_eq!(page.sections.len(), 2);
        assert_eq!(
            (page.sections[0].title.as_str(), page.sections[0].layout),
            ("Albums", Layout::Cards)
        );
        assert_eq!(page.sections[0].items[0].browse_id, "MPREb_album");
        let track = &page.sections[1].items[0];
        assert_eq!(page.sections[1].items.len(), 2);
        assert_eq!(
            (track.title.as_str(), track.subtitle.as_str(), track.duration.as_str()),
            ("Kuzu Kuzu", "Tarkan", "3:52")
        );
        assert_eq!(
            (track.video_id.as_str(), track.artist_id.as_str(), track.thumb.as_str()),
            ("abcdefghijk", "UCartist", "large")
        );
        assert_eq!(page.songs().len(), 2);
    }
}
