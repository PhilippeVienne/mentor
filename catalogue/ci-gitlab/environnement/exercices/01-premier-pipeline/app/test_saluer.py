import unittest

from saluer import saluer


class TestSaluer(unittest.TestCase):
    def test_message(self):
        self.assertEqual(saluer("Camille"), "Bienvenue chez Mentor, Camille !")


if __name__ == "__main__":
    unittest.main()
