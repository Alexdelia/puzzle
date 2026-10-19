![](fileservlet?id=855239257985)To rest from his stressful last adventure, *Doctor Who* decides to attend a music recital performed by Her Majesty the Queen’s Royal Orchestra. Since he doesn’t want to be seen in public, the Doctor uses the TARDIS to channel the music directly from the Opera to his home.

But behold! The infamous [Graske](http://tardis.wikia.com/wiki/Graske) has detected an opportunity to do evil: the sound waves channelled by the TARDIS have created a space-time crack that the alien is using to travel through dimensions. He should soon reach Earth and annihilate the human race. Nothing less could have been expected from this vile creature!

The Doctor soon realizes that the only way to prevent this terrible fate is to make the TARDIS play the melodies backwards. This will reverse the polarity of the neutron flow and banish the Graske to the other side of the universe.

Unfortunately the melodies are only available on musical scores on paper and the TARDIS is not (yet) equipped with a score reader.

Your mission is to help *Doctor Who* defeat the Graske. To do so, **you have to implement an interface that can read scanned musical scores directly from paper and translate them into musical notes**.

![](fileservlet?id=798437117788)

---

The musical scores are fairly simple and only feature half notes (![](fileservlet?id=808178814416)) and quarter notes (![](fileservlet?id=808165036238)). Notes are all represented on a staff and are limited to the following musical notes (either half or quarter):

![](fileservlet?id=819288278844)

Notes are labelled using the English convention: `A` `B` `C` `D` `E` `F` and `G`

As a reminder (for those not familiar with reading music):

- On a score, a "staff" always contains 5 lines. Notes are either located accross a line or between two lines.
- The first `C` note is located on a specific segment - a ledger line - below the 5 other lines.
- The notes have tails which can either go up - until the first `A` - or down - from the second `C`. The tail of the `B` can go either way.
- The label of a note (`A`, `B`, `C`, etc.) is independant of whether the note is a half or quarter note.

---

You are provided with scanned images of the scores in **black and white** encoded in a simple, yet efficient, form of [RLE](http://en.wikipedia.org/wiki/Run-length_encoding) (Run-Length Encoding): the **DWE** (Doctor Who Encoding) algorithm.
In the DWE, consecutive pixels of the same color are encoded using a letter (`B` for black pixels, `W` for white pixels) followed by a space followed by the number of pixels of that color.

For example: `W 5 B 20 W 16` means 5 white pixels, followed by 20 black pixels, followed by 16 white pixels.

Encoding is done from top to bottom. When the image width is known, reconstructing the original image is straightforward.

Within the images, the scores and notes have various sizes. To fully understand the challenge at hand, you should check all the images from this ![](data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAsAAAAKCAYAAABi8KSDAAAABHNCSVQICAgIfAhkiAAAAAlwSFlzAAAD2gAAA9oBEbmgvAAAABl0RVh0U29mdHdhcmUAd3d3Lmlua3NjYXBlLm9yZ5vuPBoAAAANdEVYdFRpdGxlAEdvIE5leHTrLZR+AAAAFHRFWHRBdXRob3IASmFrdWIgU3RlaW5lcub79y8AAAAhdEVYdFNvdXJjZQBodHRwOi8vamltbWFjLm11c2ljaGFsbC5jemlm414AAABJdEVYdENvcHlyaWdodABQdWJsaWMgRG9tYWluIGh0dHA6Ly9jcmVhdGl2ZWNvbW1vbnMub3JnL2xpY2Vuc2VzL3B1YmxpY2RvbWFpbi9Zw/7KAAABKElEQVQYlWP4//8/AzJmYGDQFlHkPMrAwMCDLsfEgAl402r8zS28NE4xMjLKI0uwMDIyCjEwMLAhiQkx8Xz/H9SioCkpK3yYU5Q19vvr3wcZGBgYWGy9DK6Zeiix/P/PwPD//3/G//8YGESVOJnv/D7KYJylKiuh6LFaRIm1+c2935NZ1I3kGP5aHBP+9/8P3Og7/yD0yx+3GQTdPohmyQa3yusKaGFzMwr4+/8PAwPjf2ZWNmYelqsn7jMY81q+QVagZMMleJ9lJ7MwmxzDmz0y7xfM3tT14ub3DgYGBgYeBgYGASRsWz4/4Gv7ecf/oXn2zwRlmD1hQcfy////L8imMjIyfmb6yc24q+Xe7WO7L3n9+Pj/DlwSS6SoiCnxnGBgYOBDlwMAvciNHsg6YKMAAAAASUVORK5CYII=) ![](data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAsAAAAKCAYAAABi8KSDAAAABHNCSVQICAgIfAhkiAAAAAlwSFlzAAAD2gAAA9oBEbmgvAAAABl0RVh0U29mdHdhcmUAd3d3Lmlua3NjYXBlLm9yZ5vuPBoAAAANdEVYdFRpdGxlAEdvIE5leHTrLZR+AAAAFHRFWHRBdXRob3IASmFrdWIgU3RlaW5lcub79y8AAAAhdEVYdFNvdXJjZQBodHRwOi8vamltbWFjLm11c2ljaGFsbC5jemlm414AAABJdEVYdENvcHlyaWdodABQdWJsaWMgRG9tYWluIGh0dHA6Ly9jcmVhdGl2ZWNvbW1vbnMub3JnL2xpY2Vuc2VzL3B1YmxpY2RvbWFpbi9Zw/7KAAABKElEQVQYlWP4//8/AzJmYGDQFlHkPMrAwMCDLsfEgAl402r8zS28NE4xMjLKI0uwMDIyCjEwMLAhiQkx8Xz/H9SioCkpK3yYU5Q19vvr3wcZGBgYWGy9DK6Zeiix/P/PwPD//3/G//8YGESVOJnv/D7KYJylKiuh6LFaRIm1+c2935NZ1I3kGP5aHBP+9/8P3Og7/yD0yx+3GQTdPohmyQa3yusKaGFzMwr4+/8PAwPjf2ZWNmYelqsn7jMY81q+QVagZMMleJ9lJ7MwmxzDmz0y7xfM3tT14ub3DgYGBgYeBgYGASRsWz4/4Gv7ecf/oXn2zwRlmD1hQcfy////L8imMjIyfmb6yc24q+Xe7WO7L3n9+Pj/DlwSS6SoiCnxnGBgYOBDlwMAvciNHsg6YKMAAAAASUVORK5CYII=) [page](https://files.codingame.com/pub/dw/dw2-allimages.html) ![](https://lh3.googleusercontent.com/fZt36ACYlX4Ep1JFINprlSUqWmJHLogD6Jqi4p4R9HNz_lf70UIPTGd8mpNOMu3cGtFzEwpafneckAFUrPvlvDfm2gqdPnT4fsR11aBm6Nv7dzAPocRI-aQOzQ) ![](https://lh3.googleusercontent.com/fZt36ACYlX4Ep1JFINprlSUqWmJHLogD6Jqi4p4R9HNz_lf70UIPTGd8mpNOMu3cGtFzEwpafneckAFUrPvlvDfm2gqdPnT4fsR11aBm6Nv7dzAPocRI-aQOzQ). They correspond to the challenge tests further down.

**All the test cases are contained within these 12 images and if your code can process them all, then you are good to go!**

---

**INPUT:**

**Line 1:** the width **W** and height **H** of the image in pixels.

**Line 2:** the image encoded from top to bottom using the DWE algorithm: several couples of "**C L**" separated by spaces. C is the color of the pixels (either `B` for black or `W` for white), L is the number of contiguous pixels of the same color (may encompass multiple image lines).

**OUTPUT:**

Notes read from left to right separated by space characters.
**Each note is composed of two characters**. First the note itself: `A` `B` `C` `D` `E` `F` or `G`. Then its type: `H` for a half note or `Q` for a quarter note. There is no distinction between the first `C` and the second `C` (same goes for `D`, `E`, `F`, `G`).

**CONSTRAINTS:**

100 < W < 5000
70 < H < 300

Lines and tails have a width of at least 1 pixel.
Notes are separated by at least 1 pixel.
The space between two lines is at least 8 pixels wide and at least 4 times the width of a line.

**EXAMPLE:**

| **Input** 120 176 W 4090 B 100 W 20 B 100 W 20 B 100 W 20 B 100 W 1020 B 2 W 118 B 2 W 118 B 2 W 118 B 2 W 118 B 2 W 26 B 10 W 82 B 2 W 25 B 12 W 81 B 2 W 23 B 4 W 8 B 4 W 79 B 2 W 23 B 2 W 12 B 2 W 79 B 2 W 22 B 2 W 14 B 2 W 78 B 2 W 21 B 3 W 14 B 3 W 77 B 2 W 21 B 2 W 16 B 2 W 77 B 2 W 20 B 3 W 16 B 3 W 36 B 64 W 18 B 18 W 20 B 64 W 18 B 18 W 20 B 64 W 18 B 18 W 20 B 64 W 18 B 18 W 60 B 2 W 20 B 2 W 18 B 2 W 76 B 2 W 20 B 3 W 16 B 3 W 76 B 2 W 20 B 3 W 16 B 2 W 77 B 2 W 20 B 4 W 14 B 3 W 77 B 2 W 20 B 4 W 14 B 2 W 78 B 2 W 20 B 2 W 1 B 2 W 12 B 2 W 79 B 2 W 20 B 2 W 1 B 4 W 8 B 4 W 79 B 2 W 20 B 2 W 3 B 12 W 81 B 2 W 20 B 2 W 4 B 10 W 82 B 2 W 20 B 2 W 96 B 2 W 20 B 2 W 96 B 2 W 20 B 2 W 96 B 2 W 20 B 2 W 96 B 2 W 20 B 2 W 96 B 2 W 20 B 2 W 96 B 2 W 20 B 2 W 96 B 2 W 20 B 2 W 96 B 2 W 20 B 2 W 96 B 2 W 20 B 2 W 96 B 2 W 20 B 2 W 56 B 100 W 20 B 100 W 20 B 100 W 20 B 100 W 46 B 10 W 4 B 2 W 20 B 2 W 81 B 12 W 3 B 2 W 20 B 2 W 79 B 16 W 1 B 2 W 20 B 2 W 79 B 16 W 1 B 2 W 20 B 2 W 78 B 20 W 20 B 2 W 77 B 21 W 20 B 2 W 77 B 21 W 20 B 2 W 76 B 22 W 20 B 2 W 76 B 22 W 20 B 2 W 76 B 22 W 20 B 2 W 76 B 22 W 20 B 2 W 76 B 22 W 20 B 2 W 76 B 22 W 20 B 2 W 76 B 22 W 20 B 2 W 77 B 20 W 21 B 2 W 77 B 20 W 21 B 2 W 78 B 18 W 22 B 2 W 79 B 16 W 23 B 2 W 79 B 16 W 23 B 2 W 81 B 12 W 25 B 2 W 56 B 100 W 20 B 100 W 20 B 100 W 20 B 100 W 2420 B 100 W 20 B 100 W 20 B 100 W 20 B 100 W 5050 | **Output** AQ DH ![](fileservlet?id=826996882468) |
| --- | --- |
