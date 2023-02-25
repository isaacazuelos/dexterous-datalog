# Dexterous Datalog

A [Datalog][0] implementation with no sinister secrets.

Originally written in 48 hours as part of [LangJam][5]. I just wanted a bit
more time to see if I could get a few unfinished things working.

[0]: https://en.wikipedia.org/wiki/Datalog
[5]: https://github.com/langjam/jam0004

It's Datalog but it won't let you use any identifier with letters typed with
your left hand on QWERTY. If it sees any left-handed letters, it skips them.
That means your output doesn't follow from your input, and it becomes unsound
if you use both hands!

After the jam I went and tried to clean some of this up a bit. It's still
pretty messy, but it's at least working now. There are still a lot of
under-polished loose ends.

## Build and Install

Clone this repository with `git` and use `cargo` to build and install.

```sh
cargo build
cargo run
```

## Features

Usage: `dexterous-datalog [OPTIONS] [FILENAME]`

You can have it run a file by giving it one. It'll expand the rules to produce
the full universe of facts, and print them out. It's _very dumb_ about this, so
don't try anything to big.

It'll also start up a REPL unless you pass a `--query` argument, but since
queries are incomplete, there's not much to do.

Try `--help` too for more.

## Since the Jam

I removed datafrog and just used `BTreeSet`, since I'm not too worried about it
being efficient, didn't actually use datafrog's secret sauce, and just wanted
to make it actually work.

## Jam Post-mortem

The plan was to have a full REPL, leaning on [`miette`][1] and [`chumsky`][2]
to make the UI work, and [`datafrog`][3] to do the heavy lifting for the
implementation. That's didn't quite work out.

[1]: https://github.com/zkat/miette
[2]: https://github.com/zesterer/chumsky
[3]: https://github.com/rust-lang/datafrog

I think the big take-away is that I took on more than I could finish. It's my
first time doing something like this, and the game jams I've done were all with
a team too. I spent a bit too much time on trying to be polished (repl,
diagnostics, etc) and not enough on the actual language implementation.

I probably could have saved a lot of time yesterday morning if I hadn't tried
to write a parser by hand in such an ad-hoc way the first day, but `¯\_(ツ)_/¯`

The theme was a bit tricky. I briefly explored languages in the hopes I could
write the _implementation_ in with only one hand. 

Datalog's neat, and I've wanted an excuse to play with it for a while, which
was a pretty big part of goign with this idea. Unfortunately, I didn't quite
have the time to work out how to break down rules systematically to work with
`datafrog` key-value restriction, so I just gave up about 4 hours before
submission time and try to just brute force the rules top-down by enumerating
all possible tuples and checking each sub-goal.

This was fun, but I'd probably aim smaller at the start next time, and only try
to add bells and whistles (like the repl) if there was time.

### Acknowledgements

Huge shout out to the authors of those crates: Frank McSherry (and others!),
zkat, and zesterer, and more. And JT for running this whole thing!
