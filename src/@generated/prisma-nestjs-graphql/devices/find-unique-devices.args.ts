import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { DevicesWhereUniqueInput } from './devices-where-unique.input';
import { Type } from 'class-transformer';

@ArgsType()
export class FindUniqueDevicesArgs {

    @Field(() => DevicesWhereUniqueInput, {nullable:false})
    @Type(() => DevicesWhereUniqueInput)
    where!: DevicesWhereUniqueInput;
}
