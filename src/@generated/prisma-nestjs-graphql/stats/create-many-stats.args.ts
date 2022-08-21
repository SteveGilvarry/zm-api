import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StatsCreateManyInput } from './stats-create-many.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateManyStatsArgs {

    @Field(() => [StatsCreateManyInput], {nullable:false})
    @Type(() => StatsCreateManyInput)
    data!: Array<StatsCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
