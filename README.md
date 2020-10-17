# micro_sp

## TODO
### high prioriry 
1.  [x] error handling, probably not done
2.  [x] publish complete state
3.  [x] update command vars to match measured values on refresh (raar)
4.  [x] raar: act, ref and act_ref have to have the same domain and same r#type
5.  [x] proper command line arguments
6.  [ ] add boolean variables
7.  [x] move setup to parser
8.  [ ] other command paradigm or are we done?
9.  [ ] testing
10. [x] structure and modules
11. [ ] generate dummy driver from the model
12. [x] documentation, probably not done
13. [ ] proper readme
14. [x] proper get planning result
15. [ ] include readmes to benches to describe added constraints
16. [ ] benchmarks blocksworld:
    1. [x] micro_sp inc enumerated boolean
    2. [ ] micro_sp inc pure boolean
    3. [ ] micro_sp inc pure enumeration
    4. [ ] micro_sp seq enumerated boolean
    5. [ ] micro_sp seq pure boolean
    6. [ ] micro_sp seq pure enumeration
    7. [ ] incplan
17. [ ] benchmarks barman
18. [ ] benchmarks childsnack
19. [ ] benchmarks gripper
20. [ ] benchmarks hiking
21. [ ] study conversion from a pddl model to a real world "runneble" model
22. [ ] enable other solvers beside z3
23. [ ] make a more general parser for boilerplate domain ? maybe not
24. [ ] clean the warnings
25. [ ] improve runner and don't replan after every state change
26. [ ] gui needed asap
27. [ ] show current plan in gui
28. [x] look into parsing pddls: conclusion: won't work
29. [ ] fix command lifetime
30. [x] match published data to sink when fresh timeouts
31. [ ] compositional algorithm
32. [ ] when to call the planner
33. [ ] handle estimated
34. [x] move model to models
35. [x] check if measured value is in domain
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