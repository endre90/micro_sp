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
2.  [x] move params into paramproblem
3.  [x] figure out why do tests run twice
4.  [x] invariants shouldn't be parameterized
5.  [x] don't complicate, have parameterized
6.  [x] layered (comp(param(inc)))
7.  [x] publish complete state
8.  [x] write some macros finally
9.  [x] no completestate just state that holds all
10. [ ] implemment fmt
11. [x] remove enum and bool and have only variable
12. [ ] use 'model' and 'instance'
13. [x] rename SET to ASS
14. [ ] ok, going towards making the pddl parser :(
15. [ ] think about how to parameterize modelling like the pddl people do
16. [x] update command vars to match measured values on refresh (raar)
17. [x] raar: act, ref and act_ref have to have the same domain and same r#type
18. [ ] runner: where is the state? who owns the state?
19. [x] proper command line arguments
20. [ ] need new "start/stop running" and "set goal" topics to communicate with the runner
21. [ ] the "start/stop running" should actually set the reference values     ?       
22. [x] have to add delay for dummy act             
23. [ ] add boolean variables
24. [x] add handshake kind
25. [ ] the algorithms should probably be solver agnostic
26. [x] move setup to parser
27. [ ] other command paradigm or are we done?
28. [ ] testing
29. [x] structure and modules
30. [x] generate dummy driver from the model
31. [x] documentation, probably not done
32. [ ] dummy_value should be deeply integrated
33. [ ] proper readme
34. [x] proper get planning result
35. [ ] include readmes to benches to describe added constraints
36. [x] add a timeout
37. [ ] benchmarks blocksworld:
    1. [x] micro_sp inc enumerated boolean w/ invariants
    2. [ ] micro_sp inc enumerated boolean explicit (neg-pddl)
    3. [ ] micro_sp inc pure boolean
    4. [ ] micro_sp inc pure enumeration
    5. [ ] micro_sp seq enumerated boolean
    6. [ ] micro_sp seq pure boolean
    7. [ ] micro_sp seq pure enumeration
    8. [ ] incplan
38. [ ] benchmarks barman
39. [ ] benchmarks childsnack
40. [ ] benchmarks gripper
    1. [ ] micro_sp inc enumerated boolean w/ invariants
    2. [ ] micro_sp inc enumerated boolean explicit (neg-pddl)
41. [ ] benchmarks hiking
42. [ ] study conversion from a pddl model to a real world "runneble" model
43. [ ] enable other solvers beside z3
44. [ ] make a more general parser for boilerplate domain ? maybe not
45. [ ] clean the warnings
46. [ ] improve runner and don't replan after every state change
47. [ ] gui needed asap
48. [ ] show current plan in gui
49. [x] look into parsing pddls: conclusion: won't work
50. [ ] fix command lifetime
51. [x] match published data to sink when fresh timeouts
52. [x] compositional algorithm
53. [ ] test compositional algorithm
54. [ ] when to call the planner
55. [ ] handle estimated
56. [x] move model to models
57. [x] check if measured value is in domain
58. [ ] flow graph to readme
59. [ ] maybe try to write a cdcl solver?
60. [ ] some theories? hmm...

### low priority
1. [ ] improve Qol
2. [ ] solver agnostic, yes, but...
3. [ ] could micro_sp also be planner agnostic? try with simple dfs/bfs
4. [ ] notion of costs, add optimization support?
5. [ ] nondeterminism
6. [ ] multiple goals
7. [ ] dummy in var domain default or eliminate
8. [ ] dockerize
9. [ ] maybe some quality of life for modeling
10. [ ] generate both raar and invar paradigms from a hl model?
11. [ ] explore all paths and generate graph
    
### docs tests macros status

| where  | what          |               docs |              tests |             macros |              mdocs |             mtests |            benches |
| :----- | :------------ | -----------------: | -----------------: | -----------------: | -----------------: | -----------------: | -----------------: |
| core   | compositional | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| core   | incremental   | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| core   | items         | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| core   | parameterized | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| core   | predicates    | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| models | blocksworld   | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| models | dummy_robot   | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| runner | dummy         | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| runner | node          | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| runner | publisher     | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| runner | receiver      | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| runner | sender        | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| runner | state         | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| runner | ticker        | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| utils  | core          | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| utils  | general       | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| utils  | runner        | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| main   | main          | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |

## thoughts
1. using a pddl instance and domain, to come up with a valid plan, negatives have to be
instantiated for the initial state. Otherwise init == goal in step 0. If a pddl model
should be run online with replanning, either generating these negatives in every step
would be necessary or, better, invariants would have to be included.
2. problem = model(trans and invars) + instance(init + goal)