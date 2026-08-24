mod py 'crates/pyditto'
mod js 'crates/dittojs'

default:
    @just --list

test:
    cargo test
    # TODO: Implem py and js tests
