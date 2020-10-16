# micro_sp

## TODO
### high prioriry 
1.  [x] error handling, probably not done
2.  [x] publish complete state
3.  [x] update command vars to match measured values on refresh (raar)
4.  [x] raar: act, ref and act_ref have to have the same domain and same r#type
5.  [ ] add argument to choose to only plan or to run and plan
6.  [ ] blocksworld would be an ideal example to compare bool vs. enum performance
7.  [ ] other command paradigm or are we done?
8.  [ ] testing
9.  [ ] generate dummy driver from the model
10. [x] documentation, probably not done
11. [ ] proper readme
12. [x] proper get planning result
13. [ ] benchmarks; blocks, barman, childsnack, gripper, hiking
14. [ ] improve runner and don't replan after every state change
15. [ ] gui needed asap
16. [ ] parsing pddls, look into some libs, planners
17. [ ] fix command lifetime
18. [x] match published data to sink when fresh timeouts
19. [ ] compositional algorithm
20. [ ] when to call the planner
21. [ ] gather complete state instead of measured
22. [ ] handle estimated
23. [ ] move model to models
24. [ ] make model launch choice argument
25. [x] check for measured in domain
26. [ ] flow graph to readme

### low priority
1. [ ] nondeterminism
2. [ ] multiple goals
3. [ ] dummy in var domain default or eliminate
4. [ ] dockerize
5. [ ] maybe some quality of life for modeling
6. [ ] generate both raar and invar paradigms from a hl model?
7. [ ] explore all paths and generate graph