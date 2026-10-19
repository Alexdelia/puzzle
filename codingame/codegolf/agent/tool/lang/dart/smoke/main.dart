import 'dart:io';

void main() {
  final numbers = stdin.readLineSync()!.split(' ').map(int.parse).toList();
  print(numbers[0] + numbers[1]);
}
