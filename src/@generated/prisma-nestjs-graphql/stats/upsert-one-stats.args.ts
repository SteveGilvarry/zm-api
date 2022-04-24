import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StatsWhereUniqueInput } from './stats-where-unique.input';
import { StatsCreateInput } from './stats-create.input';
import { StatsUpdateInput } from './stats-update.input';

@ArgsType()
export class UpsertOneStatsArgs {

    @Field(() => StatsWhereUniqueInput, {nullable:false})
    where!: StatsWhereUniqueInput;

    @Field(() => StatsCreateInput, {nullable:false})
    create!: StatsCreateInput;

    @Field(() => StatsUpdateInput, {nullable:false})
    update!: StatsUpdateInput;
}
