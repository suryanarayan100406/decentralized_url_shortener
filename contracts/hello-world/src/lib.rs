#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, String, Address, symbol_short};

// Structure to store URL mapping data
#[contracttype]
#[derive(Clone)]
pub struct UrlMapping {
    pub short_code: String,      // Short code identifier
    pub original_url: String,     // Full original URL
    pub creator: Address,         // Address of the creator
    pub created_at: u64,          // Timestamp of creation
    pub click_count: u64,         // Number of times accessed
}

// Counter for total URLs created
const TOTAL_URLS: Symbol = symbol_short!("TOTAL");

// Enum for mapping short codes to URL data
#[contracttype]
pub enum UrlBook {
    Url(String)  // Maps short_code -> UrlMapping
}

#[contract]
pub struct UrlShortenerContract;

#[contractimpl]
impl UrlShortenerContract {
    
    /// Creates a new short URL mapping
    /// Returns the short code assigned to the URL
    pub fn create_short_url(env: Env, short_code: String, original_url: String, creator: Address) -> String {
        // Verify the creator is authenticated
        creator.require_auth();
        
        // Check if short code already exists
        let existing = Self::get_url(env.clone(), short_code.clone());
        
        if existing.click_count > 0 || existing.created_at > 0 {
            log!(&env, "Short code already exists!");
            panic!("Short code already in use");
        }
        
        // Get current timestamp
        let timestamp = env.ledger().timestamp();
        
        // Create new URL mapping
        let url_data = UrlMapping {
            short_code: short_code.clone(),
            original_url: original_url.clone(),
            creator: creator.clone(),
            created_at: timestamp,
            click_count: 0,
        };
        
        // Store the mapping
        env.storage().instance().set(&UrlBook::Url(short_code.clone()), &url_data);
        
        // Update total count
        let mut total: u64 = env.storage().instance().get(&TOTAL_URLS).unwrap_or(0);
        total += 1;
        env.storage().instance().set(&TOTAL_URLS, &total);
        
        // Extend storage TTL
        env.storage().instance().extend_ttl(100000, 100000);
        
        log!(&env, "Short URL created: {}", short_code);
        short_code
    }
    
    /// Retrieves the original URL for a given short code
    /// Increments click counter
    pub fn resolve_url(env: Env, short_code: String) -> String {
        let key = UrlBook::Url(short_code.clone());
        
        let mut url_data: UrlMapping = env.storage().instance().get(&key).unwrap_or(UrlMapping {
            short_code: String::from_str(&env, ""),
            original_url: String::from_str(&env, "NOT_FOUND"),
            creator: Address::from_string(&String::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM")),
            created_at: 0,
            click_count: 0,
        });
        
        if url_data.created_at == 0 {
            log!(&env, "URL not found!");
            return String::from_str(&env, "NOT_FOUND");
        }
        
        // Increment click count
        url_data.click_count += 1;
        env.storage().instance().set(&key, &url_data);
        
        env.storage().instance().extend_ttl(100000, 100000);
        
        log!(&env, "URL resolved: {} clicks", url_data.click_count);
        url_data.original_url
    }
    
    /// Gets complete URL mapping data including stats
    pub fn get_url(env: Env, short_code: String) -> UrlMapping {
        let key = UrlBook::Url(short_code.clone());
        
        env.storage().instance().get(&key).unwrap_or(UrlMapping {
            short_code: String::from_str(&env, ""),
            original_url: String::from_str(&env, "NOT_FOUND"),
            creator: Address::from_string(&String::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM")),
            created_at: 0,
            click_count: 0,
        })
    }
    
    /// Returns total number of URLs created on the platform
    pub fn get_total_urls(env: Env) -> u64 {
        env.storage().instance().get(&TOTAL_URLS).unwrap_or(0)
    }
}