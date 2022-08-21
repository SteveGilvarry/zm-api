import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_DayCreateManyInput } from '../events-day/events-day-create-many.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateManyEventsDayArgs {

    @Field(() => [Events_DayCreateManyInput], {nullable:false})
    @Type(() => Events_DayCreateManyInput)
    data!: Array<Events_DayCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
