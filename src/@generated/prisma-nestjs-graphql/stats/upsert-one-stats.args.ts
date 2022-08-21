import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StatsWhereUniqueInput } from './stats-where-unique.input';
import { Type } from 'class-transformer';
import { StatsCreateInput } from './stats-create.input';
import { StatsUpdateInput } from './stats-update.input';

@ArgsType()
export class UpsertOneStatsArgs {

    @Field(() => StatsWhereUniqueInput, {nullable:false})
    @Type(() => StatsWhereUniqueInput)
    where!: StatsWhereUniqueInput;

    @Field(() => StatsCreateInput, {nullable:false})
    @Type(() => StatsCreateInput)
    create!: StatsCreateInput;

    @Field(() => StatsUpdateInput, {nullable:false})
    @Type(() => StatsUpdateInput)
    update!: StatsUpdateInput;
}
