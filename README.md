# micro_sp

## TODO
### high prioriry 
1.  [x] error handling, probably not done
2.  [x] publish complete state
3.  [x] update command vars to match measured values on refresh (raar)
4.  [x] raar: act, ref and act_ref have to have the same domain and same r#type
5.  [x] running arguments
6.  [ ] blocksworld would be an ideal example to compare bool vs. enum performance
7.  [ ] other command paradigm or are we done?
8.  [ ] testing
9.  [x] structure and modules
10. [ ] generate dummy driver from the model
11. [x] documentation, probably not done
12. [ ] proper readme
13. [x] proper get planning result
14. [ ] benchmarks blocks 
15. [ ] benchmarks barman
16. [ ] benchmarks childsnack
17. [ ] benchmarks gripper
18. [ ] benchmarks hiking
19. [ ] improve runner and don't replan after every state change
20. [ ] gui needed asap
21. [x] look into parsing pddls: conclusion: won't work
22. [ ] fix command lifetime
23. [x] match published data to sink when fresh timeouts
24. [ ] compositional algorithm
25. [ ] when to call the planner
26. [ ] handle estimated
27. [x] move model to models
28. [x] make model launch choice argument
29. [x] check for measured in domain
30. [ ] flow graph to readme

### low priority
1. [ ] improve Qol
2. [ ] nondeterminism
3. [ ] multiple goals
4. [ ] dummy in var domain default or eliminate
5. [ ] dockerize
6. [ ] maybe some quality of life for modeling
7. [ ] generate both raar and invar paradigms from a hl model?
8. [ ] explore all paths and generate graph
9. [ ] maybe move models and instances to separate crate