import { apiCall } from '/assets/js/api_call_v1.0.js';

class DashboardVTwo extends HTMLElement {
  #dropdownMenuOpen = false;
  #notificationsOpen = false;
  #adminHeaderItems = [
    {
      items: [
        {
          name: 'Home',
          component: 'home',
          icon: 'home.svg'
        },
        {
          name: 'Emoji',
          component: 'emojis',
          icon: 'home.svg'
        }
      ],
    },
    {
      name: 'Event',
      items: [
        {
          name: 'Tags',
          component: 'create-event-tag',
          icon: 'app-info.svg'
        },
        {
          titleName: 'Create Event',
          name: 'Create',
          component: 'create-event',
          icon: 'app-info.svg'
        },
        {
          name: 'List',
          component: 'event-list',
          icon: 'app-info.svg'
        }
      ],
    },
    {
      name: 'Host',
      items: [
        {
          titleName: 'Create Host',
          name: 'Create',
          component: 'create-host',
          icon: 'app-info.svg'
        },
        {
          name: 'List',
          component: 'host-list',
          icon: 'app-info.svg'
        }
      ],
    },
  ]

  #headerItems = []
  #userDetails = {}

  #stateId = 0;
  constructor() {
    super()
    this.shadow = this.attachShadow({ mode: "closed" });
    this.shadow.appendChild(this.#render())

    //event to get new notification
    this.getNewNotifications = new CustomEvent("load-new-notification", {
      bubbles: true,
      cancelable: false,
      composed: true,
      detail: true,
    });

    /**
    * @brief re-renders the workspace when user clicks the back button on browser
    */
    window.addEventListener('popstate', () => {
      let urlParams = new URLSearchParams(window.location.search)
      let headerItem = urlParams.get("h")
      let workspaceItem = urlParams.get("w")
      let component = headerItem.toLowerCase().replace("-", " ")

      if (this.#headerItems.indexOf(component) === -1) {
        window.location.href = '/dashboard'
        return
      }

      if (headerItem) {
        let target = this.shadow.querySelector(
          `.header-items [data-component-name="${headerItem}"]`
        );

        this.#changeSelectedItem(target)
      }

      if (workspaceItem) {
        let temp = workspaceItem.toLowerCase().replace("-", " ")
        let titleText = this.shadow.querySelector('.title-bar-top .comp-name')
        titleText.classList.remove('skeleton')
        titleText.innerText = temp

        this.#loadComponent(workspaceItem)
      }

      document.dispatchEvent(this.getNewNotifications)
    })

    /**
    * @brief re-renders the workspace when we want to load a component
    * in workspace thats might not be in the header list
    */
    document.addEventListener('change-workspace', (event) => {
      let details = event.detail
      let headerItem = details.h
      let workspaceItem = details.w
      let component = headerItem.toLowerCase().replace("-", " ")

      let url = '/dashboard?'
      let keys = Object.keys(details)
      for (let key of keys) {
        url += `${key}=${details[key]}&`
      }
      url = url.slice(0, -1)

      this.#stateId++
      let stateObj = { id: this.#stateId };
      window.history.pushState(stateObj, "dashboard", url);

      if (component === 'false') {
        let selectedHeader = this.shadow.querySelector('.header-container .header-items .selected')
        if (selectedHeader) {
          selectedHeader.classList.remove('selected')
          let icon = selectedHeader.querySelector('img')
          let src = icon.getAttribute('src')
          icon.setAttribute('src', src.split("-alt").join(""))
        }
      }
      else {
        if (this.#headerItems.indexOf(component) === -1) {
          window.location.href = '/dashboard'
          return
        }

        if (headerItem) {
          let target = this.shadow.querySelector(
            `.header-items [data-component-name="${headerItem}"]`
          );

          this.#changeSelectedItem(target)
        }
      }

      if (workspaceItem) {
        let temp = workspaceItem.toLowerCase().replace("-", " ")
        let titleText = this.shadow.querySelector('.title-bar-top .comp-name')
        titleText.classList.remove('skeleton')
        titleText.innerText = temp

        this.#loadComponent(workspaceItem)
      }
    })

    /**
    * @brief closes the notification container when the cross is clicked in the container
    */
    document.addEventListener("close-notification-container", () => {
      let notification = this.shadow.querySelector("notification-container");
      this.shadow.removeEventListener("click", (event) => { });
      notification.style.opacity = '0'
      setTimeout(() => {
        notification.classList.add("notification-hidden");
      }, 100);
      this.#notificationsOpen = false;
    });

    /**
    * @brief checks if there is a unread notification available and changes the
    * notification icon accordingly
    */
    document.addEventListener("notification-available", (event) => {
      let icon = this.shadow.querySelector(".notification-container img");

      if (event.detail) {
        icon.setAttribute(
          "src",
          "/components/dashboard-v-two/icon/bell-alt.svg"
        );
      } else {
        icon.setAttribute("src", "/components/dashboard-v-two/icon/bell.svg");
      }
    });

    //TODO: TEMP
    // this.#getUserDetails()
    this.#userDetails = { name: 'Fanari', role: 'Administrator' }
    this.#loadHeaderItems()
  }


  /**
    * @brief renders basic layout of the component
    */
  #render() {
    let template = document.createElement('template')
    template.innerHTML = `
    <link rel="stylesheet" href="/assets/css/reset_v1.0.css">
    <!-- <link rel="stylesheet" href="/components/components.css"> -->
    <link rel="stylesheet" href="/components/dashboard-v-two/dashboard-v-two.css">
    <div class="data-wrapper">
      <div class="title-bar-top">
        <p class="skeleton comp-name"></p>
        <div class="info-container">
          <div class="sandwich-bar" onclick="this.getRootNode().host.toggleSidebar()">
            <div class="bar bar-1"></div>
            <div class="bar bar-2"></div>
            <div class="bar bar-3"></div>
          </div>
          <a class="logo" href="/">
            <img alt="logo" src="/components/dashboard-v-two/icon/logo_full.svg">
          </a>
          <div class="notification-container primary-container" style="height:20px;width:20px;">
            <img alt="notification-icon" onclick="this.getRootNode().host.toggleNotification(event)" src="/components/dashboard-v-two/icon/bell.svg">
          </div>
          <div class="profile-container">
            <div class="profile skeleton">
              
            </div>
          </div>
        </div>
      </div>
      <div class="header-container header-hidden">
        <a href="/" class="header-icon">
          <img alt="logo" src="/components/dashboard-v-two/icon/logo_full.svg">
        </a>
        <div class="header-items">
          <div class="item-skeleton">
            <div class="icon skeleton"></div>
            <div class="text skeleton"></div>
          </div>
          <div class="item-skeleton">
            <div class="icon skeleton"></div>
            <div class="text skeleton"></div>
          </div>
          <div class="item-skeleton">
            <div class="icon skeleton"></div>
            <div class="text skeleton"></div>
          </div>
          <div class="item-skeleton">
            <div class="icon skeleton"></div>
            <div class="text skeleton"></div>
          </div>
        </div>
      </div>
      <dashboard-workspace></dashboard-workspace>
    </div>
    `
    return template.content
  }

  /**
   * @brief toggles the header item to non visible area to visible area
   * for mobile screen
   * 
   * @param {boolean} closeMode - if given true, the slider closes else toggles
   * 
   */
  toggleSidebar(closeMode) {
    let sandwichBar = this.shadow.querySelector(".sandwich-bar");
    let headerContainer = this.shadow.querySelector('.header-container')
    if (closeMode) {
      headerContainer.classList.add('header-hidden')
      sandwichBar.classList.remove("bar-open");
      return
    }

    sandwichBar.classList.toggle("bar-open");
    headerContainer.classList.toggle('header-hidden')
  }

  /**
    * @brief gets the details of a user and on success
    * calls the loadUser() and the loadHeader() function
    */
  async #getUserDetails() {
    let response = await apiCall({
      apiUrl: '/api/account/short-details',
      method: 'POST',
      reqBody: {
        user_id: localStorage.getItem('user_id')
      }
    })

    if (response.status === 'Server Down') {
      //TODO:
    }
    else if (!response.ok) {
      console.log(uniman.toast);
      uniman.toast.setNotification({
        type: 'error',
        message: response.message
      })
    }
    else {
      this.#userDetails = response.data;

      this.#loadProfile()
      this.#loadHeaderItems()
      this.#loadNotification()
    }
  }

  /**
    * @brief toggles the notification container visibility
    */
  toggleNotification(event) {
    let notification = this.shadow.querySelector("notification-container");

    if (!this.#notificationsOpen) {
      this.toggleSidebar(true)

      notification.classList.remove("notification-hidden");
      setTimeout(() => {
        notification.style.opacity = '1'
      }, 100);
      this.#notificationsOpen = true;
      setTimeout(() => {
        this.shadow.addEventListener("click", (event) => {
          let container = this.shadow.querySelector(".notification-container");

          if (!container.contains(event.target)) {
            notification.style.opacity = '0'
            setTimeout(() => {
              notification.classList.add("notification-hidden");
            }, 100);
            this.#notificationsOpen = false;
          }
        });
      }, 100);
    } else {
      this.shadow.removeEventListener("click", (event) => { });
      notification.style.opacity = '0'
      setTimeout(() => {
        notification.classList.add("notification-hidden");
      }, 100);
      this.#notificationsOpen = false;
    }
  }

  /**
    * @brief loads the notification container component
    */
  #loadNotification() {
    let wrapper = this.shadow.querySelector('.notification-container')
    let notificationContainer = document.createElement('notification-container')
    notificationContainer.classList.add('notification-hidden')
    wrapper.appendChild(notificationContainer)
  }

  /**
    * @brief renders the profile image or name after getting the
    *  user details
    */
  #loadProfile() {
    let profile = this.shadow.querySelector('.profile-container .profile')
    profile.classList.remove('skeleton')
    if (this.#userDetails.profile_image) {
      let item = document.createElement('img')
      item.setAttribute('src', `/image/${this.#userDetails.profile_image}`)
      profile.style.background = 'var(--secondary-color)'
      profile.appendChild(item)
    }
    else {
      let item = document.createElement('p')
      item.innerText = this.#userDetails.full_name.slice(0, 1)
      profile.style.background = 'var(--secondary-color)'
      profile.appendChild(item)
    }

    let profileContainer = this.shadow.querySelector('.profile-container')
    let dropDown = document.createElement('profile-dropdown')
    dropDown.classList.add('dropdown-hidden')
    dropDown.userDetails = this.#userDetails
    profileContainer.appendChild(dropDown)

    profile.addEventListener('click', () => {
      this.#toggleDropdown()
    })
  }

  /**
   * @brief toggles the profile dropdown container based on user clicking on
   * the profile or closing when clicking somewhere else
   */
  #toggleDropdown() {
    let dropdown = this.shadow.querySelector(".profile-container profile-dropdown");
    if (!this.#dropdownMenuOpen) {
      this.toggleSidebar(true)

      dropdown.classList.remove("dropdown-hidden");
      setTimeout(() => {
        dropdown.style.opacity = '1'
      }, 200);
      this.#dropdownMenuOpen = true;
      setTimeout(() => {
        this.shadow.addEventListener("click", (event) => {
          let profile = this.shadow.querySelector(".profile-container");
          if (!profile.contains(event.target)) {
            dropdown.style.opacity = '0'
            setTimeout(() => {
              dropdown.classList.add("dropdown-hidden");
            }, 200);
            this.#dropdownMenuOpen = false;
          }
        });
      }, 100);
    } else {
      this.shadow.removeEventListener("click", (event) => { });
      dropdown.style.opacity = '0'
      setTimeout(() => {
        dropdown.classList.add("dropdown-hidden");
      }, 200);
      this.#dropdownMenuOpen = false;
    }
  }

  /**
    * @brief loads the header items based on the user-details
    * and calls the headerItemClick() function for the first item
    * or if a item is specified in the url parameter
    */
  #loadHeaderItems() {
    if (this.#userDetails.role === 'Administrator') {
      this.#headerItems = this.#adminHeaderItems
    }
    // else if (this.#userDetails.role === 'Mentor') {
    //   this.#headerItems = this.#mentorHeaderItems
    // }
    // else if (this.#userDetails.role === 'User') {
    //   this.#headerItems = this.#userHeaderItems
    //   this.shadow.querySelector('.header-container').classList.add('has-bottom-content')
    // }
    else {
      throw new Error('This user role is not configured for this function')
    }

    let container = this.shadow.querySelector(".header-items");
    container.innerHTML = ''
    for (let group of this.#headerItems) {
      let groupTag = document.createElement("div");
      groupTag.classList.add("group");

      groupTag.innerHTML = `
        ${group.name == null
          ? ""
          : `
          <div class="group-title">
            <span class="group-text">${group.name}</span>

          </div>`
        }
        <div class="group-items">
        </div>
      `;

      let groupItems = groupTag.querySelector(".group-items");
      for (let item of group.items) {
        let div = document.createElement("div");
        div.classList.add("item");
        div.setAttribute("data-component-name", item.component);
        div.setAttribute("data-component-icon", item.icon);
        div.setAttribute("data-title-name", item.titleName ?? item.name);

        div.innerHTML = `
          <img src="/components/dashboard-v-two/icon/${item.icon}" alt="" class="item-logo">
          <span class="item-text">${item.name}</span>
        `;
        div.addEventListener("click", (event) =>
          this.#handleHeaderItemClick(item.component, event.currentTarget)
        );
        groupItems.appendChild(div);
      }

      container.appendChild(groupTag);
    }

    //getting url parameter
    let urlParams = new URLSearchParams(window.location.search);

    let headerItem = urlParams.get("h");
    let workspaceItem = urlParams.get("w");
    if (headerItem) {
      if (headerItem === 'false') {
        if (workspaceItem) {
          let temp = workspaceItem.toLowerCase().replace("-", " ")
          let titleText = this.shadow.querySelector('.title-bar-top .comp-name')
          titleText.classList.remove('skeleton')
          titleText.innerText = temp

          this.#loadComponent(workspaceItem)
        }
      }
      else {
        let target = this.shadow.querySelector(
          `.header-items [data-component-name="${headerItem}"]`
        );
        this.#handleReload(headerItem, workspaceItem, target, urlParams);
      }
    }
    else {
      let component = this.#headerItems[0].items[0].component;

      let target = this.shadow.querySelector(
        `.header-items [data-component-name="${component}"]`
      );

      this.#handleHeaderItemClick(component, target);
    }
  }

  /**
  * @brief based on the click calls the changeSelectedItem()
  * and loadComponent() function
  * and emits a event that will be used to fetch new notifications to render
  * 
  * @param {String} component - expected the name of the component, for which the click was
  * 
  * @param {Node} target - expected the html tag thats been clicked on
  */
  #handleReload(header, workspace, target, urlParams) {
    this.#stateId++

    let stateObj = { id: this.#stateId };
    let url = `/dashboard?`

    let keys = urlParams.keys()
    for (let key of keys) {
      url += `${key}=${urlParams.get(key)}&`
    }

    url = url.slice(0, -1)

    window.history.pushState(stateObj, "dashboard", url);

    let temp = header.toLowerCase().replace("-", " ");
    let titleText = this.shadow.querySelector('.title-bar-top .comp-name')
    titleText.classList.remove('skeleton')
    titleText.innerText = temp

    this.#changeSelectedItem(target);

    this.#loadComponent(workspace);

    this.toggleSidebar(true)
    document.dispatchEvent(this.getNewNotifications);
  }

  /**
    * @brief based on the click calls the changeSelectedItem()
    * and loadComponent() function
    * and emits a event that will be used to fetch new notifications to render
    * 
    * @param {String} component - expected the name of the component, for which the click was
    * 
    * @param {Node} target - expected the html tag thats been clicked on
    */
  #handleHeaderItemClick(component, target) {
    this.#stateId++

    let stateObj = { id: this.#stateId };
    window.history.pushState(stateObj, "dashboard", `/dashboard?h=${component}&w=${component}`);

    let name = target.getAttribute("data-title-name");
    console.log(name);

    let titleText = this.shadow.querySelector('.title-bar-top .comp-name')
    titleText.classList.remove('skeleton')
    titleText.innerText = name

    this.#changeSelectedItem(target);

    this.#loadComponent(component);

    this.toggleSidebar(true)
    document.dispatchEvent(this.getNewNotifications);
  }

  /**
    * @brief changes the clicked target to selected item design and
    *  previously selected item to a non selected design
    * 
    * @param {String} component - expected the name of the component, for which the click was
    * 
    * @param {Node} target - expected the html tag thats been clicked on
    */
  #changeSelectedItem(target) {
    //removing the previously selected
    let preSelected = this.shadow.querySelector(".header-items .item.selected");

    if (preSelected) {
      let preSelectedIcon = preSelected.querySelector(".item-logo");
      let preSelectedIconName = preSelected.getAttribute("data-component-icon");
      preSelected.classList.remove("selected");

      preSelectedIcon.setAttribute(
        "src",
        `/components/dashboard-v-two/icon/${preSelectedIconName}`
      );
    }

    //adding new selected
    if (target) {
      let targetIconName = target.getAttribute("data-component-icon");
      let targetIcon = target.querySelector(".item-logo");

      target.classList.add("selected");
      targetIcon.setAttribute(
        "src",
        `/components/dashboard-v-two/icon/${targetIconName.split(".")[0]}-alt.svg`
      );
    }
  }

  /**
    * @brief renders the component inside the dashboard workspace
    *  and passes user details to that component 
    * @param {String} component - expected the name of the component, that needs to be rendered
    */
  #loadComponent(component) {
    let workspace = this.shadow.querySelector("dashboard-workspace");

    // customComponent loader
    let customComponent = document.createElement("dash-" + component);

    //passing user details to each components
    customComponent.userDetails = this.#userDetails;

    if (component === "overview") {
      let str = ''
      for (let i = 1; i <= 50; i++) {
        str += `<slot slot="chart-${i}" name="chart-${i}"></slot>`
      }
      // chartContainer.append(donutChart)
      customComponent.innerHTML = str
    }
    /*
      Caution:
      `innerHTML` doesn’t remove the event handlers of the child nodes
      which might cause a memory leak. Use `while loop` instead!
    */
    while (workspace.firstChild) {
      workspace.removeChild(workspace.firstChild);
    }

    workspace.innerHTML = "<splash-screen></splash-screen>";
    let splashScreen = this.shadow.querySelector("splash-screen");
    setTimeout(() => {
      splashScreen.style.opacity = 0;
    }, 100);
    setTimeout(() => {
      splashScreen.remove();
    }, 700);
    workspace.appendChild(customComponent);
  }

  /**
    * @brief observers the attributes name that are mentioned
    * in the array and calls the attributeChangeCallback() function
    */
  static get observedAttributes() {
    return ["dashboard-logo"];
  }

  /**
    * @brief whatever we wat to do when a attribute changes
    */
  attributeChangedCallback(name, oldValue, newValue) {
    switch (name) {
      case "dashboard-logo":
        let logo = this.shadow.querySelector(".header-container .header-icon img")
        let logoMobile = this.shadow.querySelector(".title-bar-top .info-container .logo")
        logo.setAttribute('src', newValue)
        logoMobile.setAttribute('src', newValue)
        break;
    }
  }
}

export const dashboardVTwo = {
  mount: function () {
    customElements.define('dashboard-v-two', DashboardVTwo)
  },
  unmount: function (index) {
    index
      ? document.querySelectorAll('dashboard-v-two')[index].remove()
      : document.querySelector('dashboard-v-two').remove()
  }
}