import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntFilter } from '../prisma/int-filter.input';
import { StringFilter } from '../prisma/string-filter.input';
import { EnumDevices_TypeFilter } from '../prisma/enum-devices-type-filter.input';

@InputType()
export class DevicesWhereInput {

    @Field(() => [DevicesWhereInput], {nullable:true})
    AND?: Array<DevicesWhereInput>;

    @Field(() => [DevicesWhereInput], {nullable:true})
    OR?: Array<DevicesWhereInput>;

    @Field(() => [DevicesWhereInput], {nullable:true})
    NOT?: Array<DevicesWhereInput>;

    @Field(() => IntFilter, {nullable:true})
    Id?: IntFilter;

    @Field(() => StringFilter, {nullable:true})
    Name?: StringFilter;

    @Field(() => EnumDevices_TypeFilter, {nullable:true})
    Type?: EnumDevices_TypeFilter;

    @Field(() => StringFilter, {nullable:true})
    KeyString?: StringFilter;
}
