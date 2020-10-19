# micro_sp


## dummy 
A dummy driver (inverse of a micro_sp node) can be launched providing the argument bla bla bla

<img src='https://g.gravizo.com/svg?
 digraph G {
   micro_sp [shape=box];
   dummy [shape=box];
   micro_sp -> ref_1;
   micro_sp -> ref_2;
   ref_1 -> dummy;
   ref_2 -> dummy;
   dummy -> act_1 -> micro_sp;
   dummy -> act_2 -> micro_sp;
   dummy -> act_ref_1 -> micro_sp;
   dummy -> act_ref_2 -> micro_sp;
 }
'/>

## TODO
### high prioriry 
1.  [x] error handling, probably not done
2.  [x] publish complete state
3.  [x] update command vars to match measured values on refresh (raar)
4.  [x] raar: act, ref and act_ref have to have the same domain and same r#type
5.  [x] proper command line arguments
6.  [ ] need new "start/stop running" and "set goal" topics to communicate with the runner
7.  [ ] the "start/stop running" should actually set the reference values     ?       
8.  [ ] have to add delay for dummy act             
9.  [ ] add boolean variables
10. [x] add handshake kind
11. [ ] the algorithms should probably be solver agnostic
12. [x] move setup to parser
13. [ ] other command paradigm or are we done?
14. [ ] testing
15. [x] structure and modules
16. [ ] generate dummy driver from the model
17. [x] documentation, probably not done
18. [ ] dummy_value should be deeply integrated
19. [ ] proper readme
20. [x] proper get planning result
21. [ ] include readmes to benches to describe added constraints
22. [ ] benchmarks blocksworld:
    1. [x] micro_sp inc enumerated boolean
    2. [ ] micro_sp inc pure boolean
    3. [ ] micro_sp inc pure enumeration
    4. [ ] micro_sp seq enumerated boolean
    5. [ ] micro_sp seq pure boolean
    6. [ ] micro_sp seq pure enumeration
    7. [ ] incplan
23. [ ] benchmarks barman
24. [ ] benchmarks childsnack
25. [ ] benchmarks gripper
26. [ ] benchmarks hiking
27. [ ] study conversion from a pddl model to a real world "runneble" model
28. [ ] enable other solvers beside z3
29. [ ] make a more general parser for boilerplate domain ? maybe not
30. [ ] clean the warnings
31. [ ] improve runner and don't replan after every state change
32. [ ] gui needed asap
33. [ ] show current plan in gui
34. [x] look into parsing pddls: conclusion: won't work
35. [ ] fix command lifetime
36. [x] match published data to sink when fresh timeouts
37. [ ] compositional algorithm
38. [ ] when to call the planner
39. [ ] handle estimated
40. [x] move model to models
41. [x] check if measured value is in domain
42. [ ] flow graph to readme

### low priority
1. [ ] improve Qol
2. [ ] nondeterminism
3. [ ] multiple goals
4. [ ] dummy in var domain default or eliminate
5. [ ] dockerize
6. [ ] maybe some quality of life for modeling
7. [ ] generate both raar and invar paradigms from a hl model?
8. [ ] explore all paths and generate graph