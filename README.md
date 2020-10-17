# micro_sp

## TODO
### high prioriry 
1.  [x] error handling, probably not done
2.  [x] publish complete state
3.  [x] update command vars to match measured values on refresh (raar)
4.  [x] raar: act, ref and act_ref have to have the same domain and same r#type
5.  [x] proper command line arguments
6.  [ ] add boolean variables
7.  [x] add handshake kind
8.  [x] move setup to parser
9.  [ ] other command paradigm or are we done?
10. [ ] testing
11. [x] structure and modules
12. [ ] generate dummy driver from the model
13. [x] documentation, probably not done
14. [ ] proper readme
15. [x] proper get planning result
16. [ ] include readmes to benches to describe added constraints
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
22. [ ] study conversion from a pddl model to a real world "runneble" model
23. [ ] enable other solvers beside z3
24. [ ] make a more general parser for boilerplate domain ? maybe not
25. [ ] clean the warnings
26. [ ] improve runner and don't replan after every state change
27. [ ] gui needed asap
28. [ ] show current plan in gui
29. [x] look into parsing pddls: conclusion: won't work
30. [ ] fix command lifetime
31. [x] match published data to sink when fresh timeouts
32. [ ] compositional algorithm
33. [ ] when to call the planner
34. [ ] handle estimated
35. [x] move model to models
36. [x] check if measured value is in domain
37. [ ] flow graph to readme

### low priority
1. [ ] improve Qol
2. [ ] nondeterminism
3. [ ] multiple goals
4. [ ] dummy in var domain default or eliminate
5. [ ] dockerize
6. [ ] maybe some quality of life for modeling
7. [ ] generate both raar and invar paradigms from a hl model?
8. [ ] explore all paths and generate graph