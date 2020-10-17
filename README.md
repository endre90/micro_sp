# micro_sp

## TODO
### high prioriry 
1.  [x] error handling, probably not done
2.  [x] publish complete state
3.  [x] update command vars to match measured values on refresh (raar)
4.  [x] raar: act, ref and act_ref have to have the same domain and same r#type
5.  [x] running arguments
6.  [ ] move models to separate crate ?
7.  [ ] add boolean variables
8.  [x] move setup to parser
9.  [ ] other command paradigm or are we done?
10. [ ] testing
11. [x] structure and modules
12. [ ] generate dummy driver from the model
13. [x] documentation, probably not done
14. [ ] proper readme
15. [x] proper get planning result
16. [ ] include readme to benches to describe added constraints
17. [ ] benchmarks blocksworld:
    1. [x] micro_sp inc enumerated boolean
    2. [ ] micro_sp inc pure boolean
    3. [ ] micro_sp inc pure enumeration
    4. [ ] micro_sp seq enumerated boolean
    5. [ ] micro_sp seq pure boolean
    6. [ ] micro_sp seq pure enumeration
    7. [ ] incplan
18. [ ] benchmarks barman
19. [ ] benchmarks childsnack
20. [ ] benchmarks gripper
21. [ ] benchmarks hiking
22. [ ] enable other solvers beside z3
23. [ ] make a more general parser for boilerplate domain ? maybe not
24. [ ] clear warnings
25. [ ] improve runner and don't replan after every state change
26. [ ] gui needed asap
27. [x] look into parsing pddls: conclusion: won't work
28. [ ] fix command lifetime
29. [x] match published data to sink when fresh timeouts
30. [ ] compositional algorithm
31. [ ] when to call the planner
32. [ ] handle estimated
33. [x] move model to models
34. [x] make model launch choice argument
35. [x] check for measured in domain
36. [ ] flow graph to readme

### low priority
1. [ ] improve Qol
2. [ ] nondeterminism
3. [ ] multiple goals
4. [ ] dummy in var domain default or eliminate
5. [ ] dockerize
6. [ ] maybe some quality of life for modeling
7. [ ] generate both raar and invar paradigms from a hl model?
8. [ ] explore all paths and generate graph