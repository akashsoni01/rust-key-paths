//! Example demonstrating compile-time errors when composing incompatible keypaths
//!
//! This file intentionally contains compile errors to show what happens
//! when you try to compose keypaths from different root types.
//! 
//! To see the errors, uncomment the marked sections below.

use keypaths_proc::Kp;

#[derive(Kp, Debug)]
#[All]
struct Person {
    name: String,
    age: u32,
}

#[derive(Kp, Debug)]
#[All]
struct Product {
    name: String,
    price: f64,
}

fn main() {
    // ✅ CORRECT: This works because both keypaths share the same root
    // Person::name_r() returns KeyPath<Person, String>
    // We can't chain further because String doesn't have keypaths
    
    // ❌ ERROR 1: Trying to chain keypaths from completely different structs
    // Person::name_r() returns KeyPath<Person, String>
    // Product::name_r() expects Product as root, not String!
    // 
    // Uncomment to see error:
    // let invalid_kp = Person::name_r()
    //     .then(Product::name_r());
    // Error: expected `String`, found `Product`
    //        expected struct `String`
    //        found struct `Product`
    
    // ❌ ERROR 2: Type mismatch - trying to use a keypath that expects
    // a different type than what the first keypath produces
    // Person::age_r() returns KeyPath<Person, u32>
    // Product::name_r() expects Product, not u32!
    //
    // Uncomment to see error:
    // let invalid_kp2 = Person::age_r()
    //     .then(Product::name_r());
    // Error: expected `u32`, found `Product`
    //        expected `u32`
    //        found struct `Product`
    
    println!("This example demonstrates compile-time type safety.");
    println!("Uncomment the error cases above to see the compiler errors.");
}

