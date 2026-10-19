let a = stdin.ReadLine().Split(' ') |> Array.map int
printfn "%d" (a.[0] + a.[1])
