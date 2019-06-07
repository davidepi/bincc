//
// Created by davide on 6/5/19.
//

#ifndef __ARCHITECTURE_HPP__
#define __ARCHITECTURE_HPP__

/**
 * \brief Enum defining the type of CPU architecture
 */
enum Architecture
{
    /**
     * \brief Unknown architecture
     *
     * An error occured during analysis or the architecture is not supported
     */
    UNKNOWN = 0,

    /**
     * \brief Intel x86 or AMD64 architecture
     */
    X86,

    /**
     * \brief armhf and aarch64 architectures
     */
     ARM
};

#endif
