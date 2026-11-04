# Draft design progress

Initial no-run compile failed E0271/E0599 because the independent legacy comparator receives borrowed covariance, while the new result accessor returns a copy. Corrected only test helper iteration to iter/flatten/copied; no assertion or production behavior changed. Initial generated harness resolution was not accepted: harness lock replaced with accepted observation harness lock renamed, preserving exact root registry closure. Expected-red qualification still pending.
