# micro_sp

## TODO
### high prioriry 
1.  [x] error handling, probably not done
2.  [x] publish complete state
3.  [x] update command vars to match measured values on refresh (raar)
4.  [x] raar: act, ref and act_ref have to have the same domain and same r#type
5.  [ ] other command paradigm or are we done?
6.  [ ] testing
7.  [ ] generate dummy driver from the model
8.  [x] documentation, probably not done
9.  [ ] proper readme
10. [x] proper get planning result
11. [ ] benchmarks to models
12. [ ] improve runner and don't replan after every state change
13. [ ] gui needed asap
14. [ ] parsing pddls, look into some libs, planners
15. [ ] fix command lifetime
16. [x] match published data to sink when fresh timeouts
17. [ ] compositional algorithm
18. [ ] when to call the planner
19. [ ] gather complete state instead of measured
20. [ ] handle estimated
21. [ ] move model to models
22. [ ] make model launch choice argument
23. [x] check for measured in domain
24. [ ] flow graph to readme

### low priority
1. [ ] nondeterminism
2. [ ] multiple goals
3. [ ] dummy in var domain default or eliminate
4. [ ] dockerize
5. [ ] maybe some quality of life for modeling
6. [ ] generate both raar and invar paradigms from a hl model?
7. [ ] explore all paths and generate graph