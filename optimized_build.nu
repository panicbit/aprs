#!/usr/bin/env nu

def main [
    multi_world: path
] {
    cargo pgo build
    target/x86_64-unknown-linux-gnu/release/aprs --only-load $multi_world
    cargo pgo optimize
}


