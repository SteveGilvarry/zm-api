import { Module } from '@nestjs/common';
import { FiltersService } from './filters.service';
import { FiltersResolver } from './filters.resolver';
import { PrismaService } from '../../prisma/prisma.service';

@Module({
  providers: [PrismaService,FiltersResolver, FiltersService]
})
export class FiltersModule {}
