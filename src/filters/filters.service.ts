import { Injectable } from '@nestjs/common';
import { FiltersCreateInput } from '../@generated/prisma-nestjs-graphql/filters/filters-create.input';
import { FiltersUpdateInput } from '../@generated/prisma-nestjs-graphql/filters/filters-update.input';
import { FiltersWhereUniqueInput } from '../@generated/prisma-nestjs-graphql/filters/filters-where-unique.input';
import { PrismaService} from '../../prisma/prisma.service';
import { Filter } from '../graphql';

@Injectable()
export class FiltersService {
  constructor(private prisma: PrismaService) {}

  create(createFilterInput: FiltersCreateInput) {
    return this.prisma.filters.create({
      data: createFilterInput
    });
  }

  findAll() {
    return this.prisma.filters.findMany();
  }

  findOne(filtersWhereUniqueInput: FiltersWhereUniqueInput) {
    return this.prisma.filters.findUnique({
      where: filtersWhereUniqueInput
    });
  }

  update(
    filtersWhereUniqueInput: FiltersWhereUniqueInput,
    updateFilterInput: FiltersUpdateInput
  ) {
    return this.prisma.filters.update({
      where: filtersWhereUniqueInput,
      data: updateFilterInput
    });
  }

  remove(filtersWhereUniqueInput: FiltersWhereUniqueInput) {
    return this.prisma.filters.delete({
      where: filtersWhereUniqueInput
    });
  }
}
