"""helper functions for selenium tests."""
from __future__ import print_function

from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC


def get_items(list_element):
    """Get item elements from a list element."""
    return sorted(p.text for p in list_element.find_elements_by_tag_name('li'))


def get_ids(driver):
    """Return a list of all ids in the page.

    Mostly for debugging.

    """
    ids = driver.find_elements_by_xpath('//*[@id]')
    out = []
    for i in ids:
        out.append(i.get_attribute('id'))
    return out


def find_id(driver, id_, timeout=2):
    """find an id with a timeout."""
    return WebDriverWait(driver, timeout).until(
        EC.presence_of_element_located((By.ID, id_)))
