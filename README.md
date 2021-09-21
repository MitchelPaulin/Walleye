# Walleye

![tests](https://github.com/MitchelPaulin/ChessEngine/actions/workflows/rust.yml/badge.svg)

Walleye is a UCI-compatible engine written using the classical alpha-beta style AI. It supports loading board positions from arbitrary FEN strings, Unicode pretty printing to the console, and utilizes quiescence search (capture extension) and MVV-LVA move ordering.

Originally this project was meant as a first introduction to rust and chess programming, but it got a bit carried away.

## Example Usage

By default, the engine launches in UCI mode and expects to be loaded into a chess GUI. \
However, you can also run some commands from the terminal, such as `-P` to watch the engine play against itself or `-T` to benchmark move generation and evaluation. 


```sh
# helpful when profiling, will also accept a FEN string
./walleye -T --depth=5
```

```bash
# start a game from a FEN string and have the engine play against itself
./walleye --fen="r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1" -P
```

![demo](./demo/demo.png)

Use `./walleye --help` for a complete list of commands.

## Play Against It

The engine should be able to be loaded into any chess GUI that supports UCI, at this time though it has only been tested with [Cute Chess](https://cutechess.com/). It is recommended you compile the engine with the `--release` option for the best performance.

## Resources

Some resources and tools I found helpful when creating this engine.

- [Chess Programming Wiki](https://www.chessprogramming.org)
- [UCI Protocol Spec](https://backscattering.de/chess/uci/)
- [FEN String Generator](http://www.netreal.de/Forsyth-Edwards-Notation/index.php)

## Issues

If you find an issue with the engine please include the `walleye_{PID}.log` file along with the report. Usually found in the same directory the engine was invoked from.

## License

Walleye is under the MIT license.
