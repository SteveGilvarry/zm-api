import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StatsCreateManyInput } from './stats-create-many.input';

@ArgsType()
export class CreateManyStatsArgs {

    @Field(() => [StatsCreateManyInput], {nullable:false})
    data!: Array<StatsCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
