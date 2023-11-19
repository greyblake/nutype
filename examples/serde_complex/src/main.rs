use nutype::nutype;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Product {
    name: Name,
    image_url: ImageUrl,
    price: Price,
}

#[nutype(
    sanitize(trim),
    validate(not_empty, char_len_max = 50),
    derive(Debug, Clone, PartialEq, AsRef, Serialize, Deserialize)
)]
struct Name(String);

#[nutype(
    sanitize(trim),
    validate(
        predicate = |url| url.starts_with("https://") && url.ends_with(".jpg")
    ),
    derive(Debug, Clone, PartialEq, AsRef, Serialize, Deserialize)
)]
struct ImageUrl(String);

// Note: in the real world, you should use decimal instead of float to represent price.
#[nutype(
    validate(greater = 0.0, less = 1000_000.0),
    derive(Debug, Clone, PartialEq, AsRef, Serialize, Deserialize)
)]
struct Price(f64);

fn main() {
    {
        // Invalid because name is empty
        let json = r#"
            {
                "name": " ",
                "image_url": "https://example.com/image.jpg",
                "price": 9.99
            }
        "#;
        let res: Result<Product, _> = serde_json::from_str(json);
        let err = res.unwrap_err();
        assert!(err.to_string().contains("empty, expected valid Name"));
    }

    {
        // Invalid because image_url does not end with ".jpg"
        let json = r#"
            {
                "name": "FlySniper",
                "image_url": "https://example.com/image.png",
                "price": 9.99
            }
        "#;
        let res: Result<Product, _> = serde_json::from_str(json);
        let err = res.unwrap_err();
        assert!(err.to_string().contains("invalid, expected valid ImageUrl"));
    }

    {
        // Invalid because the price is negative
        let json = r#"
            {
                "name": "FlySniper",
                "image_url": "https://example.com/image.jpg",
                "price": -0.1
            }
        "#;
        let res: Result<Product, _> = serde_json::from_str(json);
        let err = res.unwrap_err();
        assert!(err.to_string().contains("too small, expected valid Price"));
    }

    {
        // Valid product
        let json = r#"
            {
                "name": "FlySniper\n",
                "image_url": "https://example.com/image.jpg",
                "price": 9.99
            }
        "#;
        let product: Product = serde_json::from_str(json).unwrap();
        assert_eq!(
            product,
            Product {
                name: Name::new("FlySniper").unwrap(),
                image_url: ImageUrl::new("https://example.com/image.jpg").unwrap(),
                price: Price::new(9.99).unwrap(),
            }
        )
    }
}
