# Capitol
Capitol is a Rust library for converting citation strings for United States Congressional documents of the form `<Congress><ObjectType><ObjectNumber>`, e.g., `118hr815` representing the H.R.815 in the 118th Congress, into Rust types. These Rust types can be printed as Congress.gov URLs.

This project is in its earliest stage of development, but potential future uses include:
- a polars extension for parsing GPO package ids returned via the GovInfo API into Congress.gov URLs
- a typst extension for citing Congressional legislation and hyperlinking to referenced bills

As of this writing, only citations for bills and resolutions and commitee reports from either chamber are implemented. For example, calling `Citation::parse` with the argument `118hr815` returns this Rust struct:
```rust
Citation {
    congress: Congress(118),
    chamber: Chamber::House,
    object_type: CongObjectType::HouseBill,
    number: 815,
    ver: None
}
```
Calling `to_url` on that struct return https://www.congress.gov/bill/118th-congress/house-bill/815.

## Installation

Capitol is available from crates.io.
