using Random

const Index = Int64
const Lattice = Matrix{Vector{Tuple{Index, Index}}}

struct Simulation
  h :: Float64
  t :: Float64
  n :: Float64
  spins :: Matrix{Index}
  lattice :: Lattice
end

function index_of_pos(n :: Index, pos :: Tuple{Index, Index}) :: Index
  x, y = pos
  x + n * y
end

function  pos_of_index(n :: Index, i :: Index) :: Tuple{Index, Index}
  i = i - 1
  x = rem(i, n)
  y = (i - x) / n

  (x + 1, y + 1)
end

function pot_neighbours(n :: Index, x :: Index, y :: Index) :: Vector{Index}
  local out = Vector{Index}()
  for dx in [-1 0 1]
    for dy in [-1 0 1]
      println("dx: ", dx, " dy: ", dy)
      if (dx, dy) === (0, 0)
        continue
      end

      push!(out, index_of_pos(n, (rem(x + dx, n) + 1, rem(y + dy, n) + 1)))
    end
  end

  return out
end

function make_simulation(h :: Float64, t :: Float64, n :: Int64) :: Any
  local spins = Matrix{Index}(undef, n, n)

  for (i, _) in enumerate(spins) 
    spins[i] = rand((-1, 1))
  end

  local lattice = map(_ -> [], Lattice{}(undef, n, n))
  local tried = Dict{Tuple{Index, Index}, Bool}()

  for (i, nsi) in enumerate(lattice)
    for j in pot_neighbours(n,)
  end

  return lattice
end

