class Solution:
    def findDuplicate(self, paths: List[str]) -> List[List[str]]:
        d = {}
        for path in paths:
            path = path.split()
            for file in path[1:]:
                file = file.split('(')
                file[1] = file[1][:-1]
                if file[1] in d:
                    d[file[1]].append(path[0] + '/' + file[0])
                else:
                    d[file[1]] = [path[0] + '/' + file[0]]
        return [v for v in d.values() if len(v) > 1]
