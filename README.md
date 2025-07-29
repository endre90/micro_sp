## Micro SP (SequencePlanner)

### Redis as a state manager
#### Start a Redis instance
```
$ docker run --name my-redis -p 6379:6379 -d redis
``` 
#### Start with persistent storage
```
$ docker run --name my-redis -p 6379:6379 -d redis redis-server --save 60 1 --loglevel warning
```
#### More info on Redis here: https://hub.docker.com/_/redis

## The variables

## Code coverage with tests
```
cargo tarpaulin --out Html
```