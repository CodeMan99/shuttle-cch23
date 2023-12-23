# Christmas Code Hunt

The [2023 Christmas Code Hunt](https://console.shuttle.rs/cch).

## Build and Run

Open this repository as a [devcontainer](https://containers.dev). This will setup the environment.

```shell
$ cargo build
$ cargo shuttle run
```

This will create a [Postgres](https://hub.docker.com/_/postgres) docker container for persistance. Then the [Rocket](https://rocket.rs) application should serve on `http://127.0.0.1:8000`.

## Validation

Shuttle created the [cch23-validator](https://crates.io/crates/cch23-validator) to test solutions against. In the example, day 22 is passing all test cases.

```shell
$ cch23-validator 22
⋆｡°✩ ⋆⁺｡˚⋆˙‧₊✩₊‧˙⋆˚｡⁺⋆ ✩°｡⋆°✩ ⋆⁺｡˚⋆˙‧₊✩₊‧˙⋆˚｡⁺⋆ ✩°｡⋆
.・゜゜・・゜゜・．                .・゜゜・・゜゜・．
｡･ﾟﾟ･          SHUTTLE CCH23 VALIDATOR          ･ﾟﾟ･｡
.・゜゜・・゜゜・．                .・゜゜・・゜゜・．
⋆｡°✩ ⋆⁺｡˚⋆˙‧₊✩₊‧˙⋆˚｡⁺⋆ ✩°｡⋆°✩ ⋆⁺｡˚⋆˙‧₊✩₊‧˙⋆˚｡⁺⋆ ✩°｡⋆


Validating Challenge 22...

Task 1: completed 🎉
Core tasks completed ✅
Task 2: completed 🎉
Bonus points: 600 ✨
```

## Acknowledgements

First off, **THANK YOU** to the [Shuttle](https://www.shuttle.rs) team that made these challenges available and those who supported participants throughout! 🚀

Thank you to my fellow participants in the Christmas Code Hunt! Many awesome conversation were had. I was able to share my experience as well as learn even more about the rust ecosystem. 🦀

Finally, thanks to my wife who put up with me spending time on this project. ❤️
